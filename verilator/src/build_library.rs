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

use std::{fmt::Write, fs, process::Command};

use camino::{Utf8Path, Utf8PathBuf};
use snafu::{Whatever, prelude::*};

use crate::{
    PortDirection, VerilatedModelConfig, VerilatorRuntimeOptions,
    dpi::DpiFunction,
};

fn build_ffi_for_tracing(
    buffer: &mut String,
    top_module: &str,
) -> Result<(), Whatever> {
    writeln!(
        buffer,
        r#"
    void ffi_Verilated_traceEverOn(bool everOn) {{
        Verilated::traceEverOn(everOn);
    }}

    VerilatedVcdC* ffi_V{top_module}_open_trace(V{top_module}* top, const char* path) {{
        VerilatedVcdC* vcd = new VerilatedVcdC;
        top->trace(vcd, 99);
        vcd->open(path);
        return vcd;
    }}

    void ffi_VerilatedVcdC_dump(VerilatedVcdC* vcd, uint64_t timestamp) {{
        vcd->dump(timestamp);
    }}

    void ffi_VerilatedVcdC_open_next(VerilatedVcdC* vcd, bool increment_filename) {{
        vcd->openNext(increment_filename);
    }}

    void ffi_VerilatedVcdC_flush(VerilatedVcdC* vcd) {{
        vcd->flush();
    }}

    void ffi_VerilatedVcdC_close_and_delete(VerilatedVcdC* vcd) {{
        vcd->close();
        delete vcd;
    }}
"#
    )
    .whatever_context("Failed to format tracing FFI")?;

    Ok(())
}

/// Writes `extern "C"` C++ bindings for a Verilator model with the given name
/// (`top_module`) and signature (`ports`) to the given artifact directory
/// `artifact_directory`, returning the path to the C++ file containing the FFI
/// wrappers.
fn build_ffi(
    artifact_directory: &Utf8Path,
    top_module: &str,
    ports: &[(&str, usize, usize, PortDirection)],
    enable_tracing: bool,
) -> Result<Utf8PathBuf, Whatever> {
    let ffi_wrappers = artifact_directory.join("ffi.cpp");

    let mut buffer = String::new();

    if enable_tracing {
        buffer.push_str("#include \"verilated_vcd_c.h\"\n");
        buffer.push_str("#include <stdint.h>\n");
    }

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
            whatever!(
                Err(underlying),
                "We don't support larger than 64-bit width on ports yet because weird C linkage things"
            );
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

    if enable_tracing {
        build_ffi_for_tracing(&mut buffer, top_module).whatever_context(
            "Failed to generate FFI bindings to Verilator tracing APIs",
        )?;
    }

    writeln!(&mut buffer, "}} // extern \"C\"")
        .whatever_context("Failed to format ending brace")?;

    fs::write(&ffi_wrappers, buffer)
        .whatever_context("Failed to write FFI wrappers file")?;

    Ok(ffi_wrappers)
}

/// Sets up the DPI artifacts directory and generates DPI function bindings if
/// needed, returning:
/// 1. `Some` DPI bindings file to compile in (or `None` if there are no DPI
///    functions in the first place)
/// 2. Whether there was a regeneration of any kind
///
/// This function is a nop if `dpi_functions.is_empty()`.
fn bind_dpi_if_needed(
    top_module: &str,
    dpi_functions: &[&'static dyn DpiFunction],
    dpi_artifact_directory: &Utf8Path,
    verbose: bool,
) -> Result<(Option<Utf8PathBuf>, bool), Whatever> {
    if dpi_functions.is_empty() {
        return Ok((None, false));
    }

    let dpi_file_absolute_path = dpi_artifact_directory.join("dpi.cpp");
    // TODO: hard-coded knowledge, same verilator bug
    let dpi_file = Utf8PathBuf::from("../dpi/dpi.cpp");

    let file_code = format!(
        "#include \"svdpi.h\"
#include \"V{}__Dpi.h\"
#include <stdint.h>
{}
extern \"C\" void dpi_init_callback(void** callbacks) {{
{}
}}",
        top_module,
        dpi_functions
            .iter()
            .map(|dpi_function| {
                let name = dpi_function.name();
                let signature = dpi_function
                    .signature()
                    .iter()
                    .map(|(name, ty)| format!("{} {}", ty, name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let arguments = dpi_function
                    .signature()
                    .iter()
                    .map(|(name, _)| name.to_owned())
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "static void (*rust_{})({});
extern \"C\" void {}({}) {{
    rust_{}({});
}}",
                    name, signature, name, signature, name, arguments
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        dpi_functions
            .iter()
            .enumerate()
            .map(|(i, dpi_function)| {
                let signature = dpi_function
                    .signature()
                    .iter()
                    .map(|(name, ty)| format!("{} {}", ty, name))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "   rust_{} = ( void(*)({}) ) callbacks[{}];",
                    dpi_function.name(),
                    signature,
                    i
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    );

    // only rebuild if there's been a change
    if fs::read_to_string(&dpi_file_absolute_path)
        .map(|current_file_code| current_file_code == file_code)
        .unwrap_or(false)
    {
        if verbose {
            log::info!("| Skipping regeneration of DPI due to no changes");
        }
        return Ok((Some(dpi_file), false));
    }

    if verbose {
        log::info!("| Generating DPI bindings");
    }

    fs::write(dpi_artifact_directory.join("dpi.cpp"), file_code)
        .whatever_context(format!(
            "Failed to write DPI function wrapper code to {}",
            dpi_file_absolute_path
        ))?;

    Ok((Some(dpi_file), true))
}

/// Returns `Ok(true)` when the library doesn't exist or if any Verilog source
/// file has been modified after last building the library.
fn needs_verilator_rebuild(
    source_files: &[Utf8PathBuf],
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
/// Finally, we invoke `verilator` and return the library path as well as
/// whether the library was rebuilt.
///
/// This function is not thread-safe; the `artifact_directory` must be guarded.
#[allow(clippy::too_many_arguments)]
pub fn build_library(
    source_files: &[Utf8PathBuf],
    include_directories: &[Utf8PathBuf],
    dpi_functions: &[&'static dyn DpiFunction],
    top_module: &str,
    ports: &[(&str, usize, usize, PortDirection)],
    artifact_directory: &Utf8Path,
    options: &VerilatorRuntimeOptions,
    config: &VerilatedModelConfig,
    verbose: bool,
    on_rebuild: impl FnOnce() -> Result<(), Whatever>,
) -> Result<(Utf8PathBuf, bool), Whatever> {
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
    let library_name = format!("marlin_V{}", top_module);
    let library_path =
        verilator_artifact_directory.join(format!("lib{}.so", library_name));

    let (dpi_file, dpi_rebuilt) = bind_dpi_if_needed(
        top_module,
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
        if verbose {
            log::info!(
                "| Skipping rebuild of verilated model due to no changes"
            );
        }
        return Ok((library_path, false));
    }

    on_rebuild()?;

    let _ffi_wrappers = build_ffi(
        &ffi_artifact_directory,
        top_module,
        ports,
        config.enable_tracing,
    )
    .whatever_context("Failed to build FFI wrappers")?;

    // bug in verilator#5226 means the directory must be relative to -Mdir
    let ffi_wrappers = Utf8Path::new("../ffi/ffi.cpp");

    let mut verilator_command = Command::new(&options.verilator_executable);
    verilator_command
        .args(["--cc", "-sv", "-j", "0", "--build"])
        .args(["-CFLAGS", "-shared -fpic"])
        .args(["--lib-create", &library_name])
        .args(["--Mdir", verilator_artifact_directory.as_str()])
        .args(["--top-module", top_module])
        .args(source_files)
        .arg(ffi_wrappers);
    for include_directory in include_directories {
        verilator_command.arg(format!("-I{}", include_directory));
    }
    if let Some(dpi_file) = dpi_file {
        verilator_command.arg(dpi_file);
    }
    if config.verilator_optimization != 0 {
        let level = config.verilator_optimization;
        if (1..=3).contains(&level) {
            verilator_command.arg(format!("-O{}", level));
        } else {
            whatever!("Invalid Verilator optimization level: {}", level);
        }
    }
    for ignored_warning in &config.ignored_warnings {
        verilator_command.arg(format!("-Wno-{}", ignored_warning));
    }
    if config.enable_tracing {
        verilator_command.arg("--trace");
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

    Ok((library_path, true))
}
