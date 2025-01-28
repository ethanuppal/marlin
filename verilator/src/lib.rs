// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::{hash_map::Entry, HashMap},
    ffi::{OsStr, OsString},
    fmt::Write,
    fs,
    process::Command,
};

use camino::{Utf8Path, Utf8PathBuf};
use libloading::Library;
use snafu::{whatever, ResultExt, Whatever};

/// Verilator-defined types for C FFI.
pub mod types {
    /// From the Verilator documentation: "Data representing 'bit' of 1-8 packed
    /// bits."
    pub type CData = u8;

    /// From the Verilator documentation: "Data representing 'bit' of 9-16
    /// packed bits"
    pub type SData = u16;

    /// From the Verilator documentation: "Data representing 'bit' of 17-32
    /// packed bits."
    pub type IData = u32;

    /// From the Verilator documentation: "Data representing 'bit' of 33-64
    /// packed bits."
    pub type QData = u64;

    /// From the Verilator documentation: "Data representing one element of
    /// WData array."
    pub type EData = u32;

    /// From the Verilator documentation: "Data representing >64 packed bits
    /// (used as pointer)."
    pub type WData = EData;
}

/// <https://www.digikey.com/en/maker/blogs/2024/verilog-ports-part-7-of-our-verilog-journey>
pub enum PortDirection {
    Input,
    Output,
    Inout,
}

/// You should not implement this `trait` manually. Instead, use a procedural
/// macro like `#[verilog(...)]` to derive it for you.
pub trait VerilatedModel {
    /// The source-level name of the module.
    fn name() -> &'static str;

    /// The path of the module's definition.
    fn source_path() -> &'static str;

    /// The module's interface.
    fn ports() -> &'static [(&'static str, usize, usize, PortDirection)];

    /// Use [`VerilatorRuntime::create_model`] or similar function for another
    /// runtime.
    fn init_from(library: &Library) -> Self;
}

/// `DpiFunction(c_name, c_function_code, rust_function_code)`.
pub struct DpiFunction(pub &'static str, pub &'static str, pub &'static str);

/// Optional configuration for creating a [`VerilatorRuntime`]. Usually, you can
/// just use [`VerilatorRuntimeOptions::default()`].
pub struct VerilatorRuntimeOptions {
    /// The name of the `verilator` executable, interpreted in some way by the
    /// OS/shell.
    pub verilator_executable: OsString,

    /// If `None`, there will be no optimization. If a value from `0` to `3`
    /// inclusive, the flag `-O<level>` will be passed. Enabling will slow
    /// compilation times.
    pub verilator_optimization: Option<usize>,

    /// Whether verilator should always be invoked instead of only when the
    /// source files or DPI functions change.
    pub force_verilator_rebuild: bool,

    /// The name of the `rustc` executable, interpreted in some way by the
    /// OS/shell.
    pub rustc_executable: OsString,

    /// Whether to enable optimization when calling `rustc`. Enabling will slow
    /// compilation times.
    pub rustc_optimization: bool,
}

impl Default for VerilatorRuntimeOptions {
    fn default() -> Self {
        Self {
            verilator_executable: "verilator".into(),
            verilator_optimization: None,
            force_verilator_rebuild: false,
            rustc_executable: "rustc".into(),
            rustc_optimization: false,
        }
    }
}

/// Runtime for (System)Verilog code.
pub struct VerilatorRuntime {
    artifact_directory: Utf8PathBuf,
    source_files: Vec<Utf8PathBuf>,
    dpi_functions: Vec<DpiFunction>,
    options: VerilatorRuntimeOptions,
    /// Mapping between hardware (top, path) and Verilator implementations
    libraries: HashMap<(String, String), Library>,
    verbose: bool,
}

impl VerilatorRuntime {
    /// Creates a new runtime for instantiating (Systen)Verilog modules as Rust
    /// objects.
    pub fn new<I: IntoIterator<Item = DpiFunction>>(
        artifact_directory: &Utf8Path,
        source_files: &[&Utf8Path],
        dpi_functions: I,
        options: VerilatorRuntimeOptions,
        verbose: bool,
    ) -> Result<Self, Whatever> {
        if verbose {
            log::info!("Validating source files");
        }
        for source_file in source_files {
            if !source_file.is_file() {
                whatever!(
                    "Source file (with *relative path* {}) does not exist or is not a file",
                    source_file
                );
            }
        }

        Ok(Self {
            artifact_directory: artifact_directory.to_owned(),
            source_files: source_files
                .iter()
                .map(|path| path.to_path_buf())
                .collect(),
            dpi_functions: dpi_functions.into_iter().collect(),
            options,
            libraries: HashMap::new(),
            verbose,
        })
    }

    /// Constructs a new model. Uses lazy and incremental building for
    /// efficiency.
    pub fn create_model<M: VerilatedModel>(&mut self) -> Result<M, Whatever> {
        if M::name().chars().any(|c| c == '\\' || c == ' ') {
            whatever!("Escaped module names are not supported");
        }

        if self.verbose {
            log::info!("Validating model source file");
        }
        if !self.source_files.iter().any(|source_file| {
            match (
                source_file.canonicalize_utf8(),
                Utf8Path::new(M::source_path()).canonicalize_utf8(),
            ) {
                (Ok(lhs), Ok(rhs)) => lhs == rhs,
                _ => false,
            }
        }) {
            whatever!("Module `{}` requires source file {}, which was not provided to the runtime", M::name(), M::source_path());
        }

        if let Entry::Vacant(entry) = self
            .libraries
            .entry((M::name().to_string(), M::source_path().to_string()))
        {
            let local_artifacts_directory =
                self.artifact_directory.join(M::name());

            if self.verbose {
                log::info!("Creating artifacts directory");
            }
            fs::create_dir_all(&local_artifacts_directory)
                .whatever_context("Failed to create artifacts directory")?;

            if self.verbose {
                log::info!("Building the dynamic library with verilator");
            }
            let source_files = self
                .source_files
                .iter()
                .map(|path_buf| path_buf.as_str())
                .collect::<Vec<_>>();
            let library_path = build(
                &source_files,
                &self.dpi_functions,
                M::name(),
                M::ports(),
                &local_artifacts_directory,
                &self.options,
                self.verbose,
            )
            .whatever_context("Failed to build verilator dynamic library")?;

            if self.verbose {
                log::info!("Opening the dynamic library");
            }
            let library = unsafe { Library::new(library_path) }
                .whatever_context("Failed to load verilator dynamic library")?;
            entry.insert(library);
        }

        let library = self
            .libraries
            .get(&(M::name().to_string(), M::source_path().to_string()))
            .unwrap();

        Ok(M::init_from(library))
    }
}

// TODO: hardcoded knowledge:
// - output library is obj_dir/libV${top_module}.a
// - location of verilated.h
// - verilator library is obj_dir/libverilated.a

/// Returns a tuple of the DPI object file, DPI C wrapper, and whether it was
/// rebuilt.
fn build_dpi_if_needed(
    rustc: &OsStr,
    rustc_optimize: bool,
    dpi_functions: &[DpiFunction],
    dpi_artifact_directory: &Utf8Path,
    verbose: bool,
) -> Result<(Utf8PathBuf, Utf8PathBuf, bool), Whatever> {
    let dpi_file = dpi_artifact_directory.join("dpi.rs");
    // TODO: hard-coded knowledge
    let dpi_object_file = Utf8PathBuf::from("../dpi/dpi.o"); // dpi_file.with_extension("o");
    let dpi_c_wrappers = Utf8PathBuf::from("../dpi/wrappers.c"); // dpi_artifact_directory.join("wrappers.cpp");

    let current_file_code = dpi_functions
        .iter()
        .map(|DpiFunction(_, _, rust_code)| rust_code)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");

    let c_file_code = format!(
        "#include \"verilated.h\"\n#include <stdint.h>\n{}",
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
        return Ok((dpi_object_file, dpi_c_wrappers, false));
    }

    if verbose {
        log::info!("| Building DPI");
    }

    fs::write(dpi_artifact_directory.join("wrappers.c"), c_file_code)
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
        .args(["--emit=obj", "--crate-type=lib"])
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

    Ok((dpi_object_file, dpi_c_wrappers, true))
}

fn build_ffi(
    artifact_directory: &Utf8Path,
    top: &str,
    ports: &[(&str, usize, usize, PortDirection)],
) -> Result<Utf8PathBuf, Whatever> {
    let ffi_wrappers = artifact_directory.join("ffi.cpp");

    let mut buffer = String::new();
    writeln!(
        &mut buffer,
        r#"
#include "verilated.h"
#include "V{top}.h"

extern "C" {{
    void* ffi_new_V{top}() {{
        return new V{top}{{}};
    }}

    
    void ffi_V{top}_eval(V{top}* top) {{
        top->eval();
    }}

    void ffi_delete_V{top}(V{top}* top) {{
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
                port, top
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
    void ffi_V{top}_pin_{port}(V{top}* top, {input_type}) {{
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
    {return_type} ffi_V{top}_read_{port}(V{top}* top) {{
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

fn needs_verilator_rebuild(
    source_files: &[&str],
    verilator_artifact_directory: &Utf8Path,
) -> Result<bool, Whatever> {
    if !verilator_artifact_directory.exists() {
        return Ok(true);
    }

    let Some(last_built) = fs::read_dir(verilator_artifact_directory)
        .whatever_context(format!(
            "{} exists but could not read it",
            verilator_artifact_directory
        ))?
        .flatten() // Remove failed
        .filter_map(|f| {
            if f.metadata()
                .map(|metadata| metadata.is_file())
                .unwrap_or(false)
            {
                f.metadata().unwrap().modified().ok()
            } else {
                None
            }
        })
        .max()
    else {
        return Ok(false);
    };

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

fn build(
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
    let library_name = format!("V{}_dyn", top_module);
    let library_path =
        verilator_artifact_directory.join(format!("lib{}.so", library_name));

    let (dpi_object_file, dpi_c_wrapper, dpi_rebuilt) = build_dpi_if_needed(
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
        .args(["--cc", "-sv", "--build", "-j", "0"])
        .args(["-CFLAGS", "-shared -fpic"])
        .args(["--lib-create", &library_name])
        .args(["--Mdir", verilator_artifact_directory.as_str()])
        .args(["--top-module", top_module])
        .args(source_files)
        .arg(ffi_wrappers)
        .arg(dpi_c_wrapper)
        .arg(dpi_object_file);
    if let Some(level) = options.verilator_optimization {
        if (0..=3).contains(&level) {
            verilator_command.arg(format!("-O{}", level));
        } else {
            whatever!("Invalid verilator optimization level: {}", level);
        }
    }
    if verbose {
        log::info!("| Verilator invocation: {:?}", verilator_command);
    }
    let verilator_output = verilator_command
        .output()
        .whatever_context("Invocation of verilator failed")?;

    if !verilator_output.status.success() {
        whatever!(
            "Invocation of verilator failed with nonzero exit code {}\n\n--- STDOUT ---\n{}\n\n--- STDERR ---\n{}",
            verilator_output.status,
            String::from_utf8(verilator_output.stdout).unwrap_or_default(),
            String::from_utf8(verilator_output.stderr).unwrap_or_default()
        );
    }

    Ok(library_path)
}
