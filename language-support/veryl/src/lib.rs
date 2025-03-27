// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{env::current_dir, ffi::OsString, process::Command};

use camino::Utf8PathBuf;
use marlin_verilator::{
    AsVerilatedModel, VerilatorRuntime, VerilatorRuntimeOptions,
};
use snafu::{ResultExt, Whatever, whatever};

#[doc(hidden)]
pub mod __reexports {
    pub use libloading;
    pub use marlin_verilator as verilator;
}

pub mod prelude {
    pub use crate as veryl;
    pub use crate::{VerylRuntime, VerylRuntimeOptions};
    pub use marlin_veryl_macro::veryl;
}

fn search_for_veryl_toml(mut start: Utf8PathBuf) -> Option<Utf8PathBuf> {
    while !start.as_str().is_empty() {
        if start.join("Veryl.toml").is_file() {
            return Some(start.join("Veryl.toml"));
        }
        start.pop();
    }
    None
}

/// Optional configuration for creating a [`VerylRuntime`]. Usually, you can
/// just use [`VerylRuntimeOptions::default()`].
pub struct VerylRuntimeOptions {
    /// The name of the `veryl` executable, interpreted in some way by the
    /// OS/shell.
    pub veryl_executable: OsString,

    /// Whether `veryl build` should be automatically called. This switch is
    /// useful to disable when, for example, another tool has already
    /// called `veryl build`.
    pub call_veryl_build: bool,

    /// See [`VerilatorRuntimeOptions`].
    pub verilator_options: VerilatorRuntimeOptions,
}

impl Default for VerylRuntimeOptions {
    fn default() -> Self {
        Self {
            veryl_executable: "veryl".into(),
            call_veryl_build: false,
            verilator_options: VerilatorRuntimeOptions::default(),
        }
    }
}

/// Runtime for Veryl code.
pub struct VerylRuntime {
    verilator_runtime: VerilatorRuntime,
}

impl VerylRuntime {
    /// Creates a new runtime for instantiating Veryl units as Rust objects.
    /// Does NOT call `veryl build` by defaul because `veryl build` is not
    /// thread safe. You can enable this with [`VerylRuntimeOptions`] or just
    /// run it beforehand.
    pub fn new(options: VerylRuntimeOptions) -> Result<Self, Whatever> {
        if options.verilator_options.log {
            log::info!("Searching for Veryl project root");
        }
        let Some(veryl_toml_path) = search_for_veryl_toml(
            current_dir()
                .whatever_context("Failed to get current directory")?
                .try_into()
                .whatever_context(
                    "Failed to convert current directory to UTF-8",
                )?,
        ) else {
            whatever!(
                "Failed to find Veryl.toml searching from current directory"
            );
        };
        let mut veryl_project_path = veryl_toml_path;
        veryl_project_path.pop();

        if options.call_veryl_build {
            if options.verilator_options.log {
                log::info!("Invoking `veryl build` (this may take a while)");
            }
            let veryl_output = Command::new(options.veryl_executable)
                .arg("build")
                .current_dir(&veryl_project_path)
                .output()
                .whatever_context("Invocation of veryl failed")?;

            if !veryl_output.status.success() {
                whatever!(
                    "Invocation of veryl failed with {}\n\n--- STDOUT ---\n{}\n\n--- STDERR ---\n{}",
                    veryl_output.status,
                    String::from_utf8(veryl_output.stdout).unwrap_or_default(),
                    String::from_utf8(veryl_output.stderr).unwrap_or_default()
                );
            }
        }

        let mut verilog_source_files = vec![];
        for file in veryl_project_path.join("src").read_dir_utf8().whatever_context("Failed to read contents of the src/ folder under the Veryl project root")?.flatten() {
            if file.path().extension().map(|extension| extension == "sv").unwrap_or(false) {
               verilog_source_files.push(file.path().to_path_buf());
            }
        }
        let verilog_source_files_ref = verilog_source_files
            .iter()
            .map(|path_buf| path_buf.as_path())
            .collect::<Vec<_>>();

        Ok(Self {
            verilator_runtime: VerilatorRuntime::new(
                &veryl_project_path.join("dependencies/whatever"),
                &verilog_source_files_ref,
                &[],
                [],
                options.verilator_options,
            )?,
        })
    }

    /// Instantiates a new Veryl module. This function simply wraps
    /// [`VerilatorRuntime::create_model`].
    pub fn create_model<'ctx, M: AsVerilatedModel<'ctx>>(
        &'ctx self,
    ) -> Result<M, Whatever> {
        self.verilator_runtime.create_model_simple()
    }
}
