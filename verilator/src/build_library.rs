// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

//! See the documentation for [`build_library`].

// hardcoded knowledge:
// - output library is obj_dir/libV${top_module}.a
// - location of verilated.h
// - verilator library is obj_dir/libverilated.a

use std::{ffi::OsStr, fmt::Write, fs, process::Command};

use camino::{Utf8Path, Utf8PathBuf};
use snafu::{prelude::*, Whatever};

use crate::{DpiFunction, PortDirection, VerilatorRuntimeOptions};

/// Writes `extern "C"` C++ bindings for a Verilator model with the given name
/// (`top_module`) and signature (`ports`) to the given artifact directory
/// `artifact_directory`, returning the path to the C++ file containing the FFI
/// wrappers.
fn build_ffi(
    artifact_directory: &Utf8Path,
    top_module: &str,
    ports: &[(&str, usize, usize, PortDirection)],
) -> Result<Utf8PathBuf, Whatever> {
    let ffi_wrappers = artifact_directory.join("ffi.cpp");

    let mut buffer = String::new();
    writeln!(
        &mut buffer,
        r#"
#include "verilated.h"
#include "V{top_module}.h"

extern "C" {{
    void* ffi_new_V{top_module}() {{
        return new V{top_module}{{}};
    }}

    
    void ffi_V{top_module}_eval(V{top_module}* top) {{
        top->eval();
    }}

    void ffi_delete_V{top_module}(V{top_module}* top) {{
        delete top;
    }}
"#
    )
    .whatever_context("Failed to format utility FFI")?;

    for (port, msb, lsb, direction) in ports {
        let width = msb - lsb + 1;
        if width > 64 {
            let underlying = format!(
                "Port `{}` on top module `{}` was larger than 64 bits wide",
                port, top_module
            );
            whatever!(Err(underlying), "We don't support larger than 64-bit width on ports yet because weird C linkage things");
        }
        let macro_prefix = match direction {
            PortDirection::Input => "VL_IN",
            PortDirection::Output => "VL_OUT",
            PortDirection::Inout => "VL_INOUT",
        };
        let macro_suffix = if width <= 8 {
            "8"
        } else if width <= 16 {
            "16"
        } else if width <= 32 {
            ""
        } else if width <= 64 {
            "64"
        } else {
            "W"
        };
        let type_macro = |name: Option<&str>| {
            format!(
                "{}{}({}, {}, {}{})",
                macro_prefix,
                macro_suffix,
                name.unwrap_or("/* return value */"),
                msb,
                lsb,
                if width > 64 {
                    format!(", {}", (width + 31) / 32) // words are 32 bits
                                                       // according to header
                                                       // file
                } else {
                    "".into()
                }
            )
        };

        if matches!(direction, PortDirection::Input | PortDirection::Inout) {
            let input_type = type_macro(Some("new_value"));
            writeln!(
                &mut buffer,
                r#"
    void ffi_V{top_module}_pin_{port}(V{top_module}* top, {input_type}) {{
        top->{port} = new_value;
    }}
            "#
            )
            .whatever_context("Failed to format input port FFI")?;
        }

        if matches!(direction, PortDirection::Output | PortDirection::Inout) {
            let return_type = type_macro(None);
            writeln!(
                &mut buffer,
                r#"
    {return_type} ffi_V{top_module}_read_{port}(V{top_module}* top) {{
        return top->{port};
    }}
            "#
            )
            .whatever_context("Failed to format output port FFI")?;
        }
    }

    writeln!(&mut buffer, "}} // extern \"C\"")
        .whatever_context("Failed to format ending brace")?;

    fs::write(&ffi_wrappers, buffer)
        .whatever_context("Failed to write FFI wrappers file")?;

    Ok(ffi_wrappers)
}

/// Sets up the DPI artifacts directory and builds DPI function bindings if
/// needed, returning:
/// 1. `Some` tuple of the DPI artifact files to link in (or `None` if there are
///    no DPI functions in the first place)
/// 2. Whether there was a rebuild of any kind
///
/// This function is a nop if `dpi_functions.is_empty()`.
fn build_dpi_if_needed(
    top_module: &str,
    rustc: &OsStr,
    rustc_optimize: bool,
    dpi_functions: &[DpiFunction],
    dpi_artifact_directory: &Utf8Path,
    verbose: bool,
) -> Result<(Option<(Utf8PathBuf, Utf8PathBuf)>, bool), Whatever> {
    if dpi_functions.is_empty() {
        return Ok((None, false));
    }

    let dpi_file = dpi_artifact_directory.join("dpi.rs");
    // TODO: hard-coded knowledge
    let dpi_object_file = Utf8PathBuf::from("../dpi/dpi.o"); // dpi_file.with_extension("o");
    let dpi_c_wrappers = Utf8PathBuf::from("../dpi/wrappers.cpp"); // dpi_artifact_directory.join("wrappers.cpp");

    let current_file_code = dpi_functions
        .iter()
        .map(|DpiFunction(_, _, rust_code)| rust_code)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    let c_file_code = format!(
        "#include \"svdpi.h\"\n#include \"V{}__Dpi.h\"\n#include <stdint.h>\n{}",
        top_module,
        dpi_functions
            .iter()
            .map(|DpiFunction(_, c_code, _)| c_code)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );

    // only rebuild if there's been a change
    if fs::read_to_string(&dpi_file)
        .map(|file_code| file_code == current_file_code)
        .unwrap_or(false)
    {
        if verbose {
            log::info!("| Skipping rebuild of DPI due to no changes");
        }
        return Ok((Some((dpi_object_file, dpi_c_wrappers)), false));
    }

    if verbose {
        log::info!("| Building DPI");
    }

    fs::write(dpi_artifact_directory.join("wrappers.cpp"), c_file_code)
        .whatever_context(format!(
            "Failed to write DPI function wrapper code to {}",
            dpi_c_wrappers
        ))?;
    fs::write(&dpi_file, current_file_code).whatever_context(format!(
        "Failed to write DPI function code to {}",
        dpi_file
    ))?;

    let mut rustc_command = Command::new(rustc);
    rustc_command
        .args(["--emit=obj", "--crate-type=cdylib"])
        .args(["--edition", "2021"])
        .arg(
            dpi_file
                .components()
                .last()
                .expect("We just added dpi.rs to the end..."),
        )
        .current_dir(dpi_artifact_directory);
    if rustc_optimize {
        rustc_command.arg("-O");
    }
    if verbose {
        log::info!("  | rustc invocation: {:?}", rustc_command);
    }
    let rustc_output = rustc_command
        .output()
        .whatever_context("Invocation of verilator failed")?;

    if !rustc_output.status.success() {
        whatever!(
            "Invocation of rustc failed with nonzero exit code {}\n\n--- STDOUT ---\n{}\n\n--- STDERR ---\n{}",
            rustc_output.status,
            String::from_utf8(rustc_output.stdout).unwrap_or_default(),
            String::from_utf8(rustc_output.stderr).unwrap_or_default()
        );
    }

    Ok((Some((dpi_object_file, dpi_c_wrappers)), true))
}

/// Returns `Ok(true)` when the library doesn't exist or if any Verilog source
/// file has been modified after last building the library.
fn needs_verilator_rebuild(
    source_files: &[&str],
    library_path: &Utf8Path,
) -> Result<bool, Whatever> {
    if !library_path.exists() {
        return Ok(true);
    }

    let last_built = fs::metadata(library_path)
        .whatever_context(format!(
            "Failed to read file metadata for dynamic library {}",
            library_path
        ))?
        .modified()
        .whatever_context(format!(
            "Failed to determine last-modified time for dynamic library {}",
            library_path
        ))?;

    for source_file in source_files {
        let last_edited = fs::metadata(source_file)
            .whatever_context(format!(
                "Failed to read file metadata for source file {}",
                source_file
            ))?
            .modified()
            .whatever_context(format!(
                "Failed to determine last-modified time for source file {}",
                source_file
            ))?;
        if last_edited > last_built {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Builds a dynamic library using Verilator serving as the runtime for the
/// specified Verilog module. If DPI functions are given, `rustc` compiles them
/// before they are linked with the library.
///
/// First, we set up the artifact directories (let us assume the top-level
/// directory is called "artifacts"):
/// ```text
/// artifacts/
/// ├─ ffi/
/// ├─ obj_dir/
/// ├─ dpi/
/// ```
///
/// If there are any DPI functions, we (re)build them (see
/// [`build_dpi_if_needed`]). It is important that this function is a nop when
/// there no DPI functions because invoking `rustc` takes a long time.
///
/// Then, if the DPI files were rebuilt, any Verilog source code has been
/// edited, or the `options` force rebuilding, we proceed in (re)building the
/// dynamic library. Otherwise, the function returns the library path
/// immediately here.
///
/// Next, the FFI wrappers are rebuilt (although we could probably be smarter
/// about this and only rebuild if the module's source file was edited).
///
/// Finally, we invoke `verilator` and return the library path.
pub fn build_library(
    source_files: &[&str],
    dpi_functions: &[DpiFunction],
    top_module: &str,
    ports: &[(&str, usize, usize, PortDirection)],
    artifact_directory: &Utf8Path,
    options: &VerilatorRuntimeOptions,
    verbose: bool,
) -> Result<Utf8PathBuf, Whatever> {
    if verbose {
        log::info!("| Preparing artifacts directory");
    }

    let ffi_artifact_directory = artifact_directory.join("ffi");
    fs::create_dir_all(&ffi_artifact_directory).whatever_context(
        "Failed to create ffi/ subdirectory under artifacts directory",
    )?;
    let verilator_artifact_directory = artifact_directory.join("obj_dir");
    let dpi_artifact_directory = artifact_directory.join("dpi");
    fs::create_dir_all(&dpi_artifact_directory).whatever_context(
        "Failed to create dpi/ subdirectory under artifacts directory",
    )?;
    let library_name = format!("dumbname_V{}", top_module);
    let library_path =
        verilator_artifact_directory.join(format!("lib{}.so", library_name));

    let (dpi_artifacts, dpi_rebuilt) = build_dpi_if_needed(
        top_module,
        &options.rustc_executable,
        options.rustc_optimization,
        dpi_functions,
        &dpi_artifact_directory,
        verbose,
    )
    .whatever_context("Failed to build DPI functions")?;

    if !options.force_verilator_rebuild
        && (!needs_verilator_rebuild(
            source_files,
            &verilator_artifact_directory,
        )
        .whatever_context("Failed to check if artifacts need rebuilding")?
            && !dpi_rebuilt)
    {
        log::info!("| Skipping rebuild of verilated model due to no changes");
        return Ok(library_path);
    }

    let _ffi_wrappers = build_ffi(&ffi_artifact_directory, top_module, ports)
        .whatever_context("Failed to build FFI wrappers")?;

    // bug in verilator#5226 means the directory must be relative to -Mdir
    let ffi_wrappers = Utf8Path::new("../ffi/ffi.cpp");

    let mut verilator_command = Command::new(&options.verilator_executable);
    verilator_command
        .args(["--cc", "-sv", "-j", "0"])
        .args(["-CFLAGS", "-shared -fpic"])
        .args(["--lib-create", &library_name])
        .args(["--Mdir", verilator_artifact_directory.as_str()])
        .args(["--top-module", top_module])
        .args(source_files)
        .arg(ffi_wrappers);
    if let Some((dpi_object_file, dpi_c_wrapper)) = dpi_artifacts {
        verilator_command
            .args(["-CFLAGS", dpi_object_file.as_str()])
            .arg(dpi_c_wrapper);
    }
    if let Some(level) = options.verilator_optimization {
        if (0..=3).contains(&level) {
            verilator_command.arg(format!("-O{}", level));
        } else {
            whatever!("Invalid Verilator optimization level: {}", level);
        }
    }
    if verbose {
        log::info!("| Verilator invocation: {:?}", verilator_command);
    }
    let verilator_output = verilator_command
        .output()
        .whatever_context("Invocation of Verilator failed")?;

    if !verilator_output.status.success() {
        whatever!(
            "Invocation of verilator failed with nonzero exit code {}\n\n--- STDOUT ---\n{}\n\n--- STDERR ---\n{}",
            verilator_output.status,
            String::from_utf8(verilator_output.stdout).unwrap_or_default(),
            String::from_utf8(verilator_output.stderr).unwrap_or_default()
        );
    }

    let verilator_makefile_filename =
        Utf8PathBuf::from(format!("V{}.mk", top_module));
    let verilator_makefile_path =
        verilator_artifact_directory.join(&verilator_makefile_filename);
    let verilator_makefile_contents = fs::read_to_string(
        &verilator_makefile_path,
    )
    .whatever_context(format!(
        "Failed to read Verilator-generated Makefile {}",
        verilator_makefile_path
    ))?;
    let verilator_makefile_contents = format!(
        "VK_USER_OBJS += ../dpi/dpi.o\n\n{}",
        verilator_makefile_contents
    );
    fs::write(&verilator_makefile_path, verilator_makefile_contents)
        .whatever_context(format!(
            "Failed to update Verilator-generated Makefile {}",
            verilator_makefile_path
        ))?;

    let mut make_command = Command::new(&options.make_executable);
    make_command
        .args(["-f", verilator_makefile_filename.as_str()])
        .current_dir(verilator_artifact_directory);
    if verbose {
        log::info!("| Make invocation: {:?}", make_command);
    }
    let make_output = make_command
        .output()
        .whatever_context("Invocation of Make failed")?;

    if !make_output.status.success() {
        whatever!(
            "Invocation of make failed with nonzero exit code {}\n\n--- STDOUT ---\n{}\n\n--- STDERR ---\n{}",
            make_output.status,
            String::from_utf8(make_output.stdout).unwrap_or_default(),
            String::from_utf8(make_output.stderr).unwrap_or_default()
        );
    }

    Ok(library_path)
}
