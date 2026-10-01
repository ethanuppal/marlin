// Copyright (C) 2026 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{cell::RefCell, collections::HashMap, marker::PhantomData, path::PathBuf};

use boxcar::Vec as BoxcarVec;
use libloading::Library;
use marlin_verilator_stable::core::PortDirection;

#[derive(PartialEq, Eq, Hash, Clone)]
struct LibraryArenaKey {
    name: String,
    source_path: String,
    hash: u64,
}

#[derive(Clone, Copy)]
enum BuildTarget {
    Linux,
    MacOS,
}

/// Uses [`env::consts::OS`], so it is cheap to call from a proc macro.
fn detect_os() -> Result<BuildTarget, Whatever> {
    match env::consts::OS {
        "linux" => Ok(BuildTarget::Linux),
        "macos" | "apple" => Ok(BuildTarget::MacOS),
        _ => whatever!("Unknown OS"),
    }
}

/// Runtime for (System)Verilog code.
pub struct Runtime<Impl, Config> {
    artifact_directory: PathBuf,
    build_target: BuildTarget,
    source_files: Vec<PathBuf>,
    include_directories: Vec<PathBuf>,
    /// Mapping between hardware (top, path) and arena index of Verilator
    /// implementations.
    library_map: RefCell<HashMap<LibraryArenaKey, usize>>,
    /// Verilator implementations arena.
    library_arena: BoxcarVec<Library>,

    inner: Impl,
    _generic: PhantomData<Config>
}

impl<Impl, Config> Runtime<Impl, Config> {
/// Invokes verilator to build a dynamic library for the Verilog module
/// named `name` defined in the file `source_path` and with signature
/// `ports`.
///
/// If the library is already cached for the given module name/source path
/// pair, then it is returned immediately.
///
/// It is required that the `ports` signature matches a subset of the ports
/// defined on the Verilog module exactly.
///
/// If `self.options.force_verilator_rebuild`, then the library will always
/// be rebuilt. Otherwise, it is only rebuilt on (a conservative
/// definition) of change:
///
/// - Edits to Verilog source code
/// - Edits to DPI functions
///
/// Then, if this is the first time building the library, and there are DPI
/// functions, the library will be initialized with the DPI functions.
///
/// See [`build_library::build_library`] for more information.
///
/// # Safety
///
/// This function is thread-safe.
fn build_or_retrieve_library(
    &self,
    name: &str,
    source_path: &str,
    ports: &[(&str, usize, usize, PortDirection)],
    config: &Config,
) -> Result<&Library, Whatever> {
    if !self.source_files.iter().any(|source_file| {
        match (
            source_file.canonicalize(),
            Path::new(source_path).canonicalize(),
        ) {
            (Ok(lhs), Ok(rhs)) => lhs == rhs,
            _ => false,
        }
    }) {
        whatever!(
            "Module `{}` requires source file {}, which was not provided to the runtime",
            name,
            source_path
        );
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

    let mangled_name = mangle_verilator_name_hack(name)
        .whatever_context("Failed to mangle module name")?;

    let mut hasher = hash::DefaultHasher::new();
    ports.hash(&mut hasher);
    config.hash(&mut hasher);
    let library_key = LibraryArenaKey {
        name: name.to_owned(),
        source_path: source_path.to_owned(),
        hash: hasher.finish(),
    };

    let library_idx = match self
        .library_map
        .borrow_mut()
        .entry(library_key.clone())
    {
        Entry::Occupied(entry) => *entry.get(),
        Entry::Vacant(entry) => {
            let local_directory_name = format!(
                "{}_{}_{}",
                mangled_name,
                source_path.replace("_", "__").replace("/", "_"),
                library_key.hash
            );
            let local_artifacts_directory =
                self.artifact_directory.join(&local_directory_name);

            fs::create_dir_all(&local_artifacts_directory)
                    .whatever_context(format!(
                        "Failed to create artifacts directory {local_artifacts_directory}",
                    ))?;

            //eprintln_nocapture!(
            //    "on thread {:?}",
            //    std::thread::current().id()
            //)?;

            if !THREAD_LOCKS_PER_BUILD_DIR
                .contains_key(&local_artifacts_directory)
            {
                THREAD_LOCKS_PER_BUILD_DIR.insert(
                    local_artifacts_directory.clone(),
                    Default::default(),
                );
            }
            let thread_mutex = THREAD_LOCKS_PER_BUILD_DIR
                .get(&local_artifacts_directory)
                .expect("We just inserted if it didn't exist");

            let _thread_lock = if let Ok(_thread_lock) = thread_mutex.try_lock()
            {
                //eprintln_nocapture!(
                //    "thread-level try lock for {:?} succeeded",
                //    std::thread::current().id()
                //)?;
                _thread_lock
            } else {
                eprintln_nocapture!(
                    "{} waiting for file lock on artifact directory",
                    "    Blocking".bold().green(),
                )?;
                let Ok(_thread_lock) = thread_mutex.lock() else {
                    whatever!(
                        "Failed to acquire thread-local lock for artifacts directory"
                    );
                };
                _thread_lock
            };

            // # Safety
            // build_library is not thread-safe, so we have to lock the
            // directory
            let lockfile = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(self.artifact_directory.join(format!("{local_directory_name}.lock")))
                    .whatever_context(
                        "Failed to open lockfile for artifacts directory (this is not the actual lock itself, it is an I/O error)",
                    )?;

            let _file_lock =
                file_guard::lock(&lockfile, file_guard::Lock::Exclusive, 0, 1)
                    .whatever_context(
                        "Failed to acquire file lock for artifacts directory",
                    )?;
            //eprintln_nocapture!(
            //    "lockfile for {:?} succeeded",
            //    std::thread::current().id()
            //)?;

            let start = Instant::now();

            let (library_path, was_rebuilt) = build_library(
                &self.source_files,
                self.build_target,
                &self.include_directories,
                &self.dpi_functions,
                name,
                mangled_name.as_ref(),
                ports,
                &local_artifacts_directory,
                &self.options,
                config,
                self.verilator_version,
                || {
                    eprintln_nocapture!(
                        "{} {}#{} ({})",
                        "   Compiling".bold().green(),
                        name,
                        library_key.hash,
                        source_path
                    )
                },
            )
            .whatever_context("Failed to build verilator dynamic library")?;

            let library = unsafe { Library::new(library_path) }
                .whatever_context("Failed to load verilator dynamic library")?;

            one_time_library_setup(
                &library,
                &self.dpi_functions,
                config.enable_tracing.is_some(),
            )?;

            let library_idx = self.library_arena.push(library);
            entry.insert(library_idx);

            let duration = start.elapsed()

            if was_rebuilt {
                eprintln_nocapture!(
                    "{} `verilator-{}` profile [{}] target in {}.{:02}s",
                    "    Finished".bold().green(),
                    if config.verilator_optimization == 0 {
                        "O0".into()
                    } else {
                        format!("O{}", config.verilator_optimization)
                    },
                    if config.verilator_optimization == 0 {
                        "unoptimized"
                    } else {
                        "optimized"
                    },
                    duration.as_secs(),
                    duration.subsec_millis() / 10
                )?;
            }

            library_idx
        }
    };

    Ok(self
        .library_arena
        .get(library_idx)
        .expect("bug: We just inserted the library"))
}
}
