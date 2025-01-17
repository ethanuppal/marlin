pub use spade_macro::spade;
pub use verilog::__reexports;

use std::{env::current_dir, process::Command};

use camino::{Utf8Path, Utf8PathBuf};
use snafu::{whatever, ResultExt, Whatever};
use verilog::{VerilatorRuntime, __reexports::verilator::VerilatedModel};

fn search_for_swim_toml(mut start: Utf8PathBuf) -> Option<Utf8PathBuf> {
    while !start.as_str().is_empty() {
        if start.join("swim.toml").is_file() {
            return Some(start.join("swim.toml"));
        }
        start.pop();
    }
    None
}

pub struct SpadeRuntime {
    verilator_runtime: VerilatorRuntime,
}

impl SpadeRuntime {
    pub fn new(
        artifact_directory: &Utf8Path,
        call_swim_build: bool,
        verbose: bool,
    ) -> Result<Self, Whatever> {
        if verbose {
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
            whatever!("Failed to find swim.toml");
        };

        if call_swim_build {
            if verbose {
                log::info!("Invoking `swim build` (this may take a while)");
            }
            let mut swim_project_path = swim_toml_path.clone();
            swim_project_path.pop();
            let swim_output = Command::new("swim")
                .arg("build")
                .current_dir(swim_project_path)
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

        let mut spade_sv_path = swim_toml_path;
        spade_sv_path.pop();
        spade_sv_path.push("build/spade.sv");

        Ok(Self {
            verilator_runtime: VerilatorRuntime::new(
                artifact_directory,
                &[&spade_sv_path],
                verbose,
            )?,
        })
    }

    pub fn create_model<M: VerilatedModel>(&mut self) -> Result<M, Whatever> {
        self.verilator_runtime.create_model()
    }
}
