// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::{hash_map::Entry, HashMap},
    fmt, fs,
};

use build_library::build_library;
use camino::{Utf8Path, Utf8PathBuf};
use dynamic::DynamicVerilatedModel;
use libloading::Library;
use snafu::{prelude::*, Whatever};

mod build_library;
pub mod dynamic;

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
#[derive(Debug, Clone, Copy)]
pub enum PortDirection {
    Input,
    Output,
    Inout,
}

impl fmt::Display for PortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortDirection::Input => "input",
            PortDirection::Output => "output",
            PortDirection::Inout => "inout",
        }
        .fmt(f)
    }
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

/// Runtime for (System)Verilog code.
pub struct VerilatorRuntime {
    artifact_directory: Utf8PathBuf,
    source_files: Vec<Utf8PathBuf>,
    /// Mapping between hardware (top, path) and Verilator implementations
    libraries: HashMap<(String, String), Library>,
    verbose: bool,
}

impl VerilatorRuntime {
    /// Creates a new runtime for instantiating (System)Verilog modules as Rust
    /// objects.
    pub fn new(
        artifact_directory: &Utf8Path,
        source_files: &[&Utf8Path],
        verbose: bool,
    ) -> Result<Self, Whatever> {
        if verbose {
            log::info!("Validating source files");
        }
        for source_file in source_files {
            if !source_file.is_file() {
                whatever!(
                    "Source file {} does not exist or is not a file. Note that if you specified relative paths, you must be in the correct directory",
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
            libraries: HashMap::new(),
            verbose,
        })
    }

    /// Constructs a new model. Uses lazy and incremental building for
    /// efficiency.
    pub fn create_model<M: VerilatedModel>(&mut self) -> Result<M, Whatever> {
        let library = self
            .build_or_retrieve_library(M::name(), M::source_path(), M::ports())
            .whatever_context(
                "Failed to build or retrieve verilator dynamic library",
            )?;

        Ok(M::init_from(library))
    }

    /// Constructs a new dynamic model. Uses lazy and incremental building for
    /// efficiency. You must guarantee the correctness of the suppplied
    /// information, namely, that `name` is precisely the name of the
    /// Verilog module, `source_path` is, when canonicalized
    /// using [`fs::canonicalize`], the relative/absolute path to the Verilog
    /// file defining the module `name`, and `ports` is a correct subset of
    /// the ports of the Verilog module.
    pub fn create_dyn_model<'ctx>(
        &'ctx mut self,
        name: &str,
        source_path: &str,
        ports: &[(&str, usize, usize, PortDirection)],
    ) -> Result<DynamicVerilatedModel<'ctx>, Whatever> {
        let library = self
            .build_or_retrieve_library(name, source_path, ports)
            .whatever_context(
                "Failed to build or retrieve verilator dynamic library",
            )?;

        let new_main: extern "C" fn() -> *mut libc::c_void =
            *unsafe { library.get(format!("ffi_new_V{name}").as_bytes()) }
                .whatever_context(format!(
                    "Failed to load constructor for module {}",
                    name
                ))?;
        let delete_main =
            *unsafe { library.get(format!("ffi_delete_V{name}").as_bytes()) }
                .whatever_context(format!(
                "Failed to load destructor for module {}",
                name
            ))?;
        let eval_main =
            *unsafe { library.get(format!("ffi_V{name}_eval").as_bytes()) }
                .whatever_context(format!(
                    "Failed to load evalulator for module {}",
                    name
                ))?;

        let main = new_main();

        let ports = ports
            .iter()
            .copied()
            .map(|(port, high, low, direction)| {
                (port.to_string(), (high - low + 1, direction))
            })
            .collect();

        Ok(DynamicVerilatedModel {
            ports,
            name: name.to_string(),
            main,
            delete_main,
            eval_main,
            library,
        })
    }

    fn build_or_retrieve_library(
        &mut self,
        name: &str,
        source_path: &str,
        ports: &[(&str, usize, usize, PortDirection)],
    ) -> Result<&Library, Whatever> {
        if name.chars().any(|c| c == '\\' || c == ' ') {
            whatever!("Escaped module names are not supported");
        }

        if self.verbose {
            log::info!("Validating model source file");
        }
        if !self.source_files.iter().any(|source_file| {
            match (
                source_file.canonicalize_utf8(),
                Utf8Path::new(source_path).canonicalize_utf8(),
            ) {
                (Ok(lhs), Ok(rhs)) => lhs == rhs,
                _ => false,
            }
        }) {
            whatever!("Module `{}` requires source file {}, which was not provided to the runtime", name, source_path);
        }

        if let Some((port, _, _, _)) =
            ports.iter().find(|(_, high, low, _)| high < low)
        {
            whatever!(
                "Port {} on module {} was specified with the high bit less than the low bit",
                port,
                name
            );
        }
        if let Some((port, _, _, _)) =
            ports.iter().find(|(_, high, low, _)| high + 1 - low > 64)
        {
            whatever!(
                "Port {} on module {} is greater than 64 bits",
                port,
                name
            );
        }

        if let Entry::Vacant(entry) = self
            .libraries
            .entry((name.to_string(), source_path.to_string()))
        {
            let local_artifacts_directory = self.artifact_directory.join(name);

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
            let library_path = build_library(
                &source_files,
                name,
                ports,
                &local_artifacts_directory,
            )
            .whatever_context("Failed to build verilator dynamic library")?;

            if self.verbose {
                log::info!("Opening the dynamic library");
            }
            let library = unsafe { Library::new(library_path) }
                .whatever_context("Failed to load verilator dynamic library")?;
            entry.insert(library);
        }

        Ok(self
            .libraries
            .get(&(name.to_string(), source_path.to_string()))
            .unwrap())
    }
}
