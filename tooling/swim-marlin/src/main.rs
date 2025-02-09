// Copyright (C) 2024 Ethan Uppal.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, version 3 of the License only.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along with
// this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{
    env::current_dir,
    fs,
    process::{Command, Output},
    sync::mpsc,
    thread::available_parallelism,
    time::Duration,
};

use argh::FromArgs;
use camino::{Utf8Path, Utf8PathBuf};
use indicatif::ProgressBar;
use owo_colors::OwoColorize;
use snafu::{whatever, OptionExt, ResultExt, Whatever};
use threadpool::ThreadPool;
use toml::toml;

const DEFAULT_TEST_DIRECTORY_NAME: &str = "tests";

/// Manage Marlin tests in Swim projects
#[derive(FromArgs)]
struct SwimMarlinCommand {
    #[argh(subcommand)]
    subcommand: Subcommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    Init(InitSubcommand),
    Add(AddSubcommand),
    Test(TestSubcommand),
    Check(CheckSubcommand),
}

/// initialize Marlin in an existing Swim project
#[derive(FromArgs)]
#[argh(subcommand, name = "init")]
struct InitSubcommand {
    /// manually specify the test directory name relative to the Swim project
    /// root
    #[argh(option, short = 'd')]
    test_directory: Option<Utf8PathBuf>,

    /// specify the name of the package in the generated Cargo.toml file
    #[argh(option)]
    crate_name: Option<String>,
}

/// add a new Marlin test
#[derive(FromArgs)]
#[argh(subcommand, name = "add")]
struct AddSubcommand {
    /// the name of the new test
    #[argh(positional)]
    test_name: String,
}

/// run Marlin tests
#[derive(FromArgs)]
#[argh(subcommand, name = "test")]
struct TestSubcommand {
    /// substring of test names to run
    #[argh(positional, default = "String::new()")]
    test_pattern: String,
}

/// check the well-formedness of the Marlin project
#[derive(FromArgs)]
#[argh(subcommand, name = "check")]
struct CheckSubcommand {}

fn run_shell_command(
    command: &mut Command,
    spawn: bool,
    spinner: Option<(&str, &str)>,
) -> Result<Output, Whatever> {
    let spinner_opt = if let Some((loading_message, loaded_message)) = spinner {
        let spinner = ProgressBar::new_spinner()
            .with_message(loading_message.to_string());
        spinner.enable_steady_tick(Duration::from_millis(100));
        Some((spinner, loaded_message))
    } else {
        None
    };
    let program = command.get_program().to_string_lossy().to_string();
    let output = if !spawn {
        command
            .output()
            .whatever_context(format!("Invocation of `{}` failed", program))?
    } else {
        command
            .spawn()
            .whatever_context(format!("Invocation of `{}` failed", program))?
            .wait_with_output()
            .whatever_context(format!(
                "Waiting for output of invoked `{}` failed",
                program
            ))?
    };

    if !output.status.success() {
        whatever!(
            "Invocation of {} failed with {}\n\n--- STDOUT ---\n{}\n\n--- STDERR ---\n{}",
            program,
            output.status,
            String::from_utf8(output.stdout).unwrap_or_default(),
            String::from_utf8(output.stderr).unwrap_or_default()
        );
    }

    if let Some((spinner, loaded_message)) = spinner_opt {
        spinner.finish_with_message(loaded_message.to_string());
    }

    Ok(output)
}

fn add_test(
    cargo_toml: &mut toml::Value,
    test_name: &str,
) -> Result<(), Whatever> {
    let test_directory = cargo_toml
            .get("package")
            .and_then(|package| package.get("metadata"))
            .and_then(|metadata| metadata.get("marlin"))
            .and_then(|marlin| marlin.get("test_directory"))
        .and_then(|test_directory| test_directory.as_str())
        .whatever_context("Failed to read `package.metadata.marlin.test_directory` as a string in Cargo.toml")?.to_string();

    let Some(table) = cargo_toml.as_table_mut() else {
        whatever!("Cargo.toml is not a table");
    };

    let Some(binaries) = table
        .entry("bin")
        .or_insert(toml::Value::Array(vec![]))
        .as_array_mut()
    else {
        whatever!("Binaries section in Cargo.toml is not an array");
    };

    if binaries.iter().any(|binary| {
        binary
            .get("name")
            .and_then(|name| name.as_str())
            .map(|name| name == test_name)
            .unwrap_or(false)
    }) {
        println!("Test {} already exists", test_name);
        return Ok(());
    }

    let binary_path =
        Utf8Path::new(&test_directory).join(format!("{}.rs", test_name));
    if binary_path.is_file() {
        whatever!(
            "File {} already exists, cannot overwrite to create test file",
            binary_path
        );
    }
    binaries.push(
        toml! {
            name = test_name
            path = (binary_path.as_str())
        }
        .into(),
    );

    fs::write(binary_path, include_str!("../resources/test_template.rs"))
        .whatever_context("Failed to initialize test code file")?;

    Ok(())
}

fn check(cargo_toml: &toml::Value) -> Result<(), Whatever> {
    let Some(marlin_metadata) = cargo_toml
        .get("package")
        .and_then(|package| package.get("metadata"))
        .and_then(|metadata| metadata.get("marlin"))
    else {
        whatever!("Missing [package.metadata.marlin] section from Cargo.toml");
    };

    if !cargo_toml
        .get("dependencies")
        .and_then(|dependencies| dependencies.as_table())
        .map(|dependencies| dependencies.contains_key("marlin"))
        .unwrap_or(false)
    {
        whatever!("Missing Marlin dependency from [dependencies] section in Cargo.toml");
    }

    let Some(test_directory) = marlin_metadata
        .get("test_directory")
        .and_then(|test_directory| test_directory.as_str())
    else {
        whatever!("Missing test_directory string under [package.metadata.marlin] in Cargo.toml");
    };

    if !Utf8Path::new(test_directory).is_dir() {
        whatever!("Test directory under [package.metadata.marlin] in Cargo.toml either does not exist or is not a directory");
    }

    Ok(())
}

fn init(
    current_directory: Utf8PathBuf,
    options: InitSubcommand,
    swim_toml: toml::Value,
) -> Result<(), Whatever> {
    let cargo_toml_path = current_directory.join("Cargo.toml");
    if cargo_toml_path.is_file() {
        let cargo_toml_contents = fs::read_to_string(&cargo_toml_path)
            .whatever_context(format!(
                "Failed to read Cargo.toml at {}",
                cargo_toml_path
            ))?;
        let cargo_toml: toml::Value = toml::from_str(&cargo_toml_contents)
            .whatever_context(format!(
                "Failed to parse Cargo.toml at {}",
                cargo_toml_path
            ))?;

        if cargo_toml
            .get("package")
            .and_then(|package| package.get("metadata"))
            .and_then(|metadata| metadata.get("marlin"))
            .is_some()
        {
            // the last step is adding marlin and other dependencies
            if cargo_toml
                .get("dependencies")
                .and_then(|dependencies| dependencies.as_table())
                .map(|dependencies| dependencies.contains_key("marlin"))
                .unwrap_or(false)
            {
                println!(
                    "Ran `swim marlin init` on an already-initialized project"
                );
                println!();
                println!("  Use `swim marlin help` for more information");
                return Ok(());
            }
        } else {
            whatever!("A Cargo.toml already exists in the project directory not created by `swim marlin init` --- remove it or manually setup Marlin");
        }
    }
    // inv: Cargo.toml does not exist

    let crate_name = options
        .crate_name
        .or_else(|| {
            swim_toml
                .get("name")
                .and_then(|name| name.as_str())
                .map(|name| format!("{}_tests", name))
        })
        .whatever_context(
            "Failed to read required `name` field in swim.toml",
        )?;

    let test_directory_name = Utf8Path::new(
        options
            .test_directory
            .as_ref()
            .map(|test_directory| test_directory.as_str())
            .unwrap_or(
                swim_toml
                    .get("simulation")
                    .and_then(|simulation| simulation.get("testbench_dir"))
                    .and_then(|testbench_directory| {
                        testbench_directory.as_str()
                    })
                    .unwrap_or(DEFAULT_TEST_DIRECTORY_NAME),
            ),
    );

    let test_directory_path = current_directory.join(test_directory_name);

    println!("  Setting up test directory");
    fs::create_dir_all(&test_directory_path).whatever_context(format!(
        "Failed to create test directory at {}",
        test_directory_path
    ))?;

    println!("  Setting up Rust stuff");
    let mut cargo_toml = toml::Value::Table(toml! {
        [workspace]

        [package]
        name = crate_name
        edition = "2021"

        [package.metadata.marlin]
        test_directory = (test_directory_name.as_str())
    });
    add_test(&mut cargo_toml, "test")
        .whatever_context("Failed to add initial test")?;
    let cargo_toml_string = toml::to_string_pretty(&cargo_toml)
        .whatever_context("Failed to format generated Cargo.toml as string")?;
    fs::write(cargo_toml_path, cargo_toml_string)
        .whatever_context("Failed to create Cargo.toml in current directory")?;

    run_shell_command(
        Command::new("cargo").args([
            "add",
            "marlin",
            "-Fmarlin/spade",
            "snafu",
            "colog",
        ]),
        false,
        Some((
            "Loading Marlin from crates.io",
            "Loaded Marlin from crates.io",
        )),
    )
    .whatever_context("Failed to add Marlin as a dependency")?;

    if let Ok(gitignore_contents) =
        fs::read_to_string(current_directory.join(".gitignore"))
    {
        fs::write(current_directory.join(".gitignore"), format!("{}\ntarget/", gitignore_contents)).whatever_context("Failed to update .gitignore to exclude the target/ build directory")?;
        println!("  Updated .gitignore")
    }

    println!("  Marlin initialized successfully!");
    println!();
    println!("  Use `swim marlin help` for more information");

    Ok(())
}

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let command: SwimMarlinCommand = argh::from_env();

    let current_directory = Utf8PathBuf::from_path_buf(
        current_dir()
            .whatever_context("Failed to determine current directory")?,
    )
    .map_err(|_| "?")
    .whatever_context("Failed to parse current directory as UTF-8")?;

    let swim_toml_path = current_directory.join("swim.toml");

    let swim_toml_contents = fs::read_to_string(&swim_toml_path)
        .whatever_context("Failed to read swim.toml in current directory")?;

    let swim_toml: toml::Value = toml::from_str(&swim_toml_contents)
        .whatever_context("Failed to parse swim.toml in current directory")?;

    match command.subcommand {
        Subcommand::Init(init_subcommand) => {
            init(current_directory, init_subcommand, swim_toml)
        }
        Subcommand::Add(add_subcommand) => {
            let cargo_toml_path = current_directory.join("Cargo.toml");
            let cargo_toml_contents = fs::read_to_string(&cargo_toml_path)
                .whatever_context("Run `swim marlin init` first")?;
            let mut cargo_toml: toml::Value = toml::from_str(
                &cargo_toml_contents,
            )
            .whatever_context(format!(
                "Failed to parse Cargo.toml at {}",
                cargo_toml_path
            ))?;

            check(&cargo_toml)?;

            add_test(&mut cargo_toml, &add_subcommand.test_name)
                .whatever_context(format!(
                    "Failed to add test {}",
                    add_subcommand.test_name
                ))
        }
        Subcommand::Test(test_subcommand) => {
            let cargo_toml_path = current_directory.join("Cargo.toml");
            let cargo_toml_contents = fs::read_to_string(&cargo_toml_path)
                .whatever_context("Run `swim marlin init` first")?;
            let cargo_toml: toml::Value = toml::from_str(&cargo_toml_contents)
                .whatever_context(format!(
                    "Failed to parse Cargo.toml at {}",
                    cargo_toml_path
                ))?;

            check(&cargo_toml)?;

            let test_directory  = cargo_toml
            .get("package")
            .and_then(|package| package.get("metadata"))
            .and_then(|metadata| metadata.get("marlin"))
            .and_then(|marlin| marlin.get("test_directory"))
                .and_then(|test_directory| test_directory.as_str())
                .map(Utf8Path::new)
                .whatever_context("Failed to read `package.metadata.marlin.test_directory` as a string in Cargo.toml")?;

            run_shell_command(
                Command::new("cargo").args(["build", "--all"]),
                true,
                None,
            )
            .whatever_context("Failed to build test code")?;

            let mut test_names = vec![];
            for test in test_directory.read_dir_utf8().whatever_context("Failed to read the test directory specified in package.metadata.marlin.test_directory")?.flatten() {
                if test.path().extension().unwrap_or("") != "rs" {
                    continue;
                }

                if let Some(test_name) = test.path().file_stem() {
                    let is_binary_in_cargo_toml = cargo_toml.get("bin").and_then(|binaries| binaries.as_array()).map(|binaries| binaries.iter().any(|binary| {
                        binary.get("name").and_then(|name| name.as_str()).map(|name| name == test_name).unwrap_or(false)
                    })).unwrap_or(false);
                    if is_binary_in_cargo_toml && test_name.contains(&test_subcommand.test_pattern) {
                        test_names.push(test_name.to_string());
                    }
                }
            }

            let _worker_count = available_parallelism()
                .map(|value| value.get())
                .unwrap_or(1);
            let worker_count = 1; // TODO: until we can make artifacts directory thread safe
            let pool = ThreadPool::new(worker_count);

            let test_count = test_names.len();
            println!(
                "{} {} test{} [{}/{}.rs] across {} thread{}",
                "     STARTING".bold().bright_cyan(),
                test_count,
                if test_count == 1 { "" } else { "s" },
                test_directory,
                if test_subcommand.test_pattern.is_empty() {
                    "*".to_string()
                } else {
                    format!("*{}*", test_subcommand.test_pattern)
                },
                worker_count,
                if worker_count == 1 { "" } else { "s" },
            );

            let (tx, rx) = mpsc::channel();

            for test_name in test_names {
                let tx = tx.clone();
                let test_directory = test_directory.to_path_buf();
                pool.execute(move || {
                    let test_path =
                        format!("{}/{}.rs", test_directory, test_name);
                    let result = match run_shell_command(
                        &mut Command::new(format!(
                            "target/debug/{}",
                            test_name
                        )),
                        false,
                        None,
                    ) {
                        Ok(_) => Ok(format!(
                            "         {} [{}]",
                            "PASS".bold().bright_green(),
                            test_path
                        )),
                        Err(error) => Err(format!(
                            "        {} [{}]\n{}",
                            "FAIL".bold().bright_red(),
                            test_path,
                            error
                        )),
                    };
                    let _ = tx.send(result);
                });
            }

            let mut failures = 0;
            for _ in 0..test_count {
                match rx.recv() {
                    Ok(result) => match result {
                        Ok(success) => println!("{}", success),
                        Err(failure) => {
                            failures += 1;
                            println!("{}", failure);
                        }
                    },
                    Err(error) => {
                        println!(
                            "      {} [<unknown test>]: {}",
                            "CLOSED".bold().on_bright_yellow(),
                            error
                        );
                    }
                }
            }

            println!(
                "{} with {} failure{}",
                "     FINISHED".bold().bright_cyan(),
                failures,
                if failures == 1 { "" } else { "s" },
            );

            if failures > 0 {
                whatever!("Exiting due to failure(s)");
            }

            Ok(())
        }
        Subcommand::Check(_check_subcommand) => {
            let cargo_toml_path = current_directory.join("Cargo.toml");
            let cargo_toml_contents = fs::read_to_string(&cargo_toml_path)
                .whatever_context("Run `swim marlin init` first")?;
            let cargo_toml: toml::Value = toml::from_str(&cargo_toml_contents)
                .whatever_context(format!(
                    "Failed to parse Cargo.toml at {}",
                    cargo_toml_path
                ))?;

            check(&cargo_toml)?;
            println!("Everything looks good!");
            Ok(())
        }
    }
}
