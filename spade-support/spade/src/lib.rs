pub use spade_macro::spade;
pub use verilog::__reexports;

use std::{env::current_dir, ffi::OsString, process::Command};

use camino::Utf8PathBuf;
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

/// Optional configuration for creating a `SpadeRuntime`. Usually, you can just
/// use `SpadeRuntimeOptions::default()`.
pub struct SpadeRuntimeOptions {
    /// The name of the `swim` executable, interpreted in some way by the
    /// OS/shell.
    pub swim_executable: OsString,

    /// Whether `swim build` should be automatically called. This switch is
    /// useful to disable when, for example, another tool has already
    /// called `swim build`.
    pub call_swim_build: bool,
}

impl Default for SpadeRuntimeOptions {
    fn default() -> Self {
        Self {
            swim_executable: "swim".into(),
            call_swim_build: true,
        }
    }
}

/// Runtime for Spade code.
pub struct SpadeRuntime {
    verilator_runtime: VerilatorRuntime,
}

impl SpadeRuntime {
    /// Creates a new runtime for instantiating Spade units as Rust objects.
    pub fn new(
        options: SpadeRuntimeOptions,
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
            whatever!(
                "Failed to find swim.toml searching from current directory"
            );
        };
        let mut swim_project_path = swim_toml_path;
        swim_project_path.pop();

        if options.call_swim_build {
            if verbose {
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

        Ok(Self {
            verilator_runtime: VerilatorRuntime::new(
                // https://discord.com/channels/962274366043873301/962296357018828822/1332274022280466503
                &swim_project_path.join("build/thirdparty"),
                &[&spade_sv_path],
                verbose,
            )?,
        })
    }

    /// Instantiates a new Spade unit. This function simply wraps
    /// [`VerilatorRuntime::create_model`].
    pub fn create_model<M: VerilatedModel>(&mut self) -> Result<M, Whatever> {
        self.verilator_runtime.create_model()
    }
}
