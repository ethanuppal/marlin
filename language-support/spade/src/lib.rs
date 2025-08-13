// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

//! Spade integration for Marlin.

use std::{env::current_dir, ffi::OsString, fs, process::Command};

use camino::{Utf8Path, Utf8PathBuf};
use marlin_verilator::{
    AsVerilatedModel, VerilatedModelConfig, VerilatorRuntime,
    VerilatorRuntimeOptions,
};
use snafu::{ResultExt, Whatever, whatever};

#[doc(hidden)]
pub mod __reexports {
    pub use libloading;
    pub use marlin_verilator as verilator;
}

pub mod prelude {
    pub use crate as spade;
    pub use crate::{SpadeRuntime, SpadeRuntimeOptions};
    pub use marlin_spade_macro::spade;
}

fn search_for_swim_toml(mut start: Utf8PathBuf) -> Option<Utf8PathBuf> {
    while start.parent().is_some() {
        if start.join("swim.toml").is_file() {
            return Some(start.join("swim.toml"));
        }
        start.pop();
    }
    None
}

/// Optional configuration for creating a [`SpadeRuntime`]. Usually, you can
/// just use [`SpadeRuntimeOptions::default()`].
pub struct SpadeRuntimeOptions {
    /// The name of the `swim` executable, interpreted in some way by the
    /// OS/shell.
    pub swim_executable: OsString,

    /// Whether `swim build` should be automatically called. This switch is
    /// useful to disable when, for example, another tool has already
    /// called `swim build`.
    pub call_swim_build: bool,

    /// See [`VerilatorRuntimeOptions`].
    pub verilator_options: VerilatorRuntimeOptions,
}

impl Default for SpadeRuntimeOptions {
    fn default() -> Self {
        Self {
            swim_executable: "swim".into(),
            call_swim_build: false,
            verilator_options: VerilatorRuntimeOptions::default(),
        }
    }
}

impl SpadeRuntimeOptions {
    /// The same as the [`Default`] implementation except that the log crate is
    /// used.
    pub fn default_logging() -> Self {
        Self {
            verilator_options: VerilatorRuntimeOptions::default_logging(),
            ..Default::default()
        }
    }
}

/// Optional configuration for creating an [`AsVerilatedModel`]. Usually, you
/// can just use [`SpadeModelConfig::default()`].
#[derive(Default)]
pub struct SpadeModelConfig {
    /// See [`VerilatedModelConfig`].
    pub verilator_config: VerilatedModelConfig,
}

/// Runtime for Spade code.
pub struct SpadeRuntime {
    verilator_runtime: VerilatorRuntime,
}

impl SpadeRuntime {
    /// Creates a new runtime for instantiating Spade units as Rust objects.
    /// Does NOT call `swim build` by defaul because `swim build` is not
    /// thread safe. You can enable this with [`SwimRuntimeOptions`] or just
    /// run it beforehand.
    pub fn new(options: SpadeRuntimeOptions) -> Result<Self, Whatever> {
        if options.verilator_options.log {
            log::info!("Searching for swim project root");
        }
        let Some(swim_toml_path) = search_for_swim_toml(
            current_dir()
                .whatever_context("Failed to get current directory")?
                .try_into()
                .whatever_context(
                    "Failed to convert current directory to UTF-8",
                )?,
        ) else {
            whatever!(
                "Failed to find swim.toml searching from current directory"
            );
        };
        let mut swim_project_path = swim_toml_path.clone();
        swim_project_path.pop();

        if options.call_swim_build {
            if options.verilator_options.log {
                log::info!("Invoking `swim build` (this may take a while)");
            }
            let swim_output = Command::new(options.swim_executable)
                .arg("build")
                .current_dir(&swim_project_path)
                .output()
                .whatever_context("Invocation of swim failed")?;

            if !swim_output.status.success() {
                whatever!(
                    "Invocation of swim failed with nonzero exit code {}\n\n--- STDOUT ---\n{}\n\n--- STDERR ---\n{}",
                    swim_output.status,
                    String::from_utf8(swim_output.stdout).unwrap_or_default(),
                    String::from_utf8(swim_output.stderr).unwrap_or_default()
                );
            }
        }

        let spade_sv_path = swim_project_path.join("build/spade.sv");

        let swim_toml_contents = fs::read_to_string(&swim_toml_path)
            .whatever_context(format!(
                "Failed to read contents of swim.toml at {swim_toml_path}"
            ))?;
        let swim_toml: toml::Value = toml::from_str(&swim_toml_contents)
            .whatever_context(
                "Failed to parse swim.toml as a valid TOML file",
            )?;

        let extra_verilog = swim_toml.get("verilog").map(|verilog| {
            (
                verilog
                    .get("sources")
                    .and_then(|sources| sources.as_array())
                    .map(|sources| {
                        let mut result = vec![];
                        for source in
                            sources.iter().flat_map(|source| source.as_str())
                        {
                            if let Ok(paths) = glob::glob(
                                swim_project_path.join(source).as_str(),
                            ) {
                                for path in paths.flatten() {
                                    if let Ok(path) =
                                        Utf8PathBuf::try_from(path)
                                    {
                                        result.push(path);
                                    }
                                }
                            }
                        }
                        result
                    }),
                verilog
                    .get("include")
                    .and_then(|include| include.as_array())
                    .map(|sources| {
                        sources
                            .iter()
                            .flat_map(|source| source.as_str())
                            .flat_map(|source| {
                                Utf8Path::new(source).canonicalize_utf8()
                            })
                            .collect::<Vec<_>>()
                    }),
            )
        });

        let mut source_files = vec![spade_sv_path.as_path()];
        let mut include_files = vec![];

        if let Some(extra_verilog) = &extra_verilog {
            if let Some(sources) = &extra_verilog.0 {
                source_files
                    .extend(sources.iter().map(|source| source.as_path()));
            }
            if let Some(include) = &extra_verilog.1 {
                include_files.extend(
                    include.iter().map(|directory| directory.as_path()),
                );
            }
        }

        Ok(Self {
            verilator_runtime: VerilatorRuntime::new(
                // https://discord.com/channels/962274366043873301/962296357018828822/1332274022280466503
                &swim_project_path.join("build/thirdparty/marlin"),
                &source_files,
                &include_files,
                [],
                options.verilator_options,
            )?,
        })
    }

    /// Instantiates a new Spade unit. This function simply wraps
    /// [`VerilatorRuntime::create_model_simple`].
    pub fn create_model_simple<'ctx, M: AsVerilatedModel<'ctx>>(
        &'ctx self,
    ) -> Result<M, Whatever> {
        self.verilator_runtime.create_model_simple()
    }

    /// Instantiates a new Spade unit. This function simply wraps
    /// [`VerilatorRuntime::create_model`].
    pub fn create_model<'ctx, M: AsVerilatedModel<'ctx>>(
        &'ctx self,
        config: SpadeModelConfig,
    ) -> Result<M, Whatever> {
        self.verilator_runtime
            .create_model(&config.verilator_config)
    }
}
