// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

//! This module implements the Verilator runtime for instantiating hardware
//! modules. The Marlin Verilator runtime supports DPI, VCDs, and dynamic
//! models in addition to standard models.
//!
//! For an example of how to use this runtime to add support for your own custom
//! HDL, see `SpadeRuntime` (under the "language-support/spade/" directory),
//! which just wraps [`VerilatorRuntime`].

use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    ffi::{self, OsString},
    fmt, fs,
    hash::{self, Hash, Hasher},
    io::Write,
    os::fd::FromRawFd,
    sync::{LazyLock, Mutex},
    time::Instant,
};

use boxcar::Vec as BoxcarVec;
use build_library::build_library;
use camino::{Utf8Path, Utf8PathBuf};
use dashmap::DashMap;
use dpi::DpiFunction;
use dynamic::DynamicVerilatedModel;
use libloading::Library;
use owo_colors::OwoColorize;
use snafu::{ResultExt, Whatever, whatever};

mod build_library;
pub mod dpi;
pub mod dynamic;
pub mod vcd;

pub use dynamic::AsDynamicVerilatedModel;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

/// Configuration for a particular [`VerilatedModel`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct VerilatedModelConfig {
    /// If `None`, there will be no optimization. If a value from `0` to `3`
    /// inclusive, the flag `-O<level>` will be passed. Enabling will slow
    /// compilation times.
    pub verilator_optimization: usize,

    /// A list of Verilator warnings to disable on the Verilog source code for
    /// this model.
    pub ignored_warnings: Vec<String>,

    /// Whether this model should be compiled with tracing support.
    pub enable_tracing: bool,
}

/// You should not implement this `trait` manually. Instead, use a procedural
/// macro like `#[verilog(...)]` to derive it for you.
pub trait AsVerilatedModel<'ctx>: 'ctx {
    /// The source-level name of the module.
    fn name() -> &'static str;

    /// The path of the module's definition.
    fn source_path() -> &'static str;

    /// The module's interface.
    fn ports() -> &'static [(&'static str, usize, usize, PortDirection)];

    #[doc(hidden)]
    fn init_from(library: &'ctx Library, tracing_enabled: bool) -> Self;

    #[doc(hidden)]
    unsafe fn model(&self) -> *mut ffi::c_void;
}

/// Optional configuration for creating a [`VerilatorRuntime`]. Usually, you can
/// just use [`VerilatorRuntimeOptions::default()`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VerilatorRuntimeOptions {
    /// The name of the `verilator` executable, interpreted in some way by the
    /// OS/shell.
    pub verilator_executable: OsString,

    /// Whether Verilator should always be invoked instead of only when the
    /// source files or DPI functions change.
    pub force_verilator_rebuild: bool,

    /// Whether to use the log crate.
    pub log: bool,
}

impl Default for VerilatorRuntimeOptions {
    fn default() -> Self {
        Self {
            verilator_executable: "verilator".into(),
            force_verilator_rebuild: false,
            log: false,
        }
    }
}

impl VerilatorRuntimeOptions {
    /// The same as the [`Default`] implementation except that the log crate is
    /// used.
    pub fn default_logging() -> Self {
        Self {
            log: true,
            ..Default::default()
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct LibraryArenaKey {
    name: String,
    source_path: String,
    hash: u64,
}

/// Runtime for (System)Verilog code.
pub struct VerilatorRuntime {
    artifact_directory: Utf8PathBuf,
    source_files: Vec<Utf8PathBuf>,
    include_directories: Vec<Utf8PathBuf>,
    dpi_functions: Vec<&'static dyn DpiFunction>,
    options: VerilatorRuntimeOptions,
    /// Mapping between hardware (top, path) and arena index of Verilator
    /// implementations
    library_map: RefCell<HashMap<LibraryArenaKey, usize>>,
    /// Verilator implementations arena
    library_arena: BoxcarVec<Library>,
    /// SAFETY: These are dropped when the runtime is dropped. They will not be
    /// "borrowed mutably" because the models created for this runtime must
    /// not outlive it and thus will be all gone before these are dropped.
    model_deallocators:
        RefCell<Vec<(*mut ffi::c_void, extern "C" fn(*mut ffi::c_void))>>,
}

impl Drop for VerilatorRuntime {
    fn drop(&mut self) {
        for (model, deallocator) in
            self.model_deallocators.borrow_mut().drain(..)
        {
            deallocator(model);
        }
    }
}

/* <Forgive me father for I have sinned> */

// TODO: make cross-platform
static STDERR: LazyLock<Mutex<fs::File>> =
    LazyLock::new(|| Mutex::new(unsafe { fs::File::from_raw_fd(2) }));

macro_rules! eprintln_nocapture {
    ($($contents:tt)*) => {{
        use snafu::ResultExt;

        writeln!(
            &mut STDERR.lock().expect("poisoned"),
            $($contents)*
        )
        .whatever_context("Failed to write to non-captured stderr")
    }};
}

#[derive(Default)]
struct ThreadLocalFileLock;

/// The file_guard handles locking across processes, but does not guarantee
/// locking between threads in one process. Thus, we have this lock to
/// synchronize threads for a given artifacts directoru.
static THREAD_LOCKS_PER_BUILD_DIR: LazyLock<
    DashMap<Utf8PathBuf, Mutex<ThreadLocalFileLock>>,
> = LazyLock::new(DashMap::default);

/* </Forgive me father for I have sinned> */

fn one_time_library_setup(
    library: &Library,
    dpi_functions: &[&'static dyn DpiFunction],
    tracing_enabled: bool,
    options: &VerilatorRuntimeOptions,
) -> Result<(), Whatever> {
    if !dpi_functions.is_empty() {
        let dpi_init_callback: extern "C" fn(*const *const ffi::c_void) =
            *unsafe { library.get(b"dpi_init_callback") }
                .whatever_context("Failed to load DPI initializer")?;

        // order is important here. the function pointers will be
        // initialized in the same order that they
        // appear in the DPI array --- this is to match how the C
        // initialization code was constructed in `build_library`.
        let function_pointers = dpi_functions
            .iter()
            .map(|dpi_function| dpi_function.pointer())
            .collect::<Vec<_>>();

        (dpi_init_callback)(function_pointers.as_ptr_range().start);

        if options.log {
            log::info!("Initialized DPI functions");
        }
    }

    if tracing_enabled {
        let trace_ever_on_callback: extern "C" fn(bool) =
            *unsafe { library.get(b"ffi_Verilated_traceEverOn") }
                .whatever_context(
                    "Model was not configured with tracing enabled",
                )?;
        trace_ever_on_callback(true);

        if options.log {
            log::info!("Initialized VCD tracing");
        }
    }

    Ok(())
}

impl VerilatorRuntime {
    /// Creates a new runtime for instantiating (System)Verilog modules as Rust
    /// objects.
    pub fn new(
        artifact_directory: &Utf8Path,
        source_files: &[&Utf8Path],
        include_directories: &[&Utf8Path],
        dpi_functions: impl IntoIterator<Item = &'static dyn DpiFunction>,
        options: VerilatorRuntimeOptions,
    ) -> Result<Self, Whatever> {
        if options.log {
            log::info!("Validating source files");
        }
        for source_file in source_files {
            if !source_file.is_file() {
                whatever!(
                    "Source file {} does not exist or is not a file. Note that if it's a relative path, you must be in the correct directory",
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
            include_directories: include_directories
                .iter()
                .map(|path| path.to_path_buf())
                .collect(),
            dpi_functions: dpi_functions.into_iter().collect(),
            options,
            library_map: RefCell::new(HashMap::new()),
            library_arena: BoxcarVec::new(),
            model_deallocators: RefCell::new(vec![]),
        })
    }

    /// Constructs a new model. Uses lazy and incremental building for
    /// efficiency.
    ///
    /// See also: [`VerilatorRuntime::create_dyn_model`]
    pub fn create_model_simple<'ctx, M: AsVerilatedModel<'ctx>>(
        &'ctx self,
    ) -> Result<M, Whatever> {
        self.create_model(&VerilatedModelConfig::default())
    }

    /// Constructs a new model. Uses lazy and incremental building for
    /// efficiency.
    ///
    /// See also: [`VerilatorRuntime::create_dyn_model`]
    pub fn create_model<'ctx, M: AsVerilatedModel<'ctx>>(
        &'ctx self,
        config: &VerilatedModelConfig,
    ) -> Result<M, Whatever> {
        let library = self
            .build_or_retrieve_library(
                M::name(),
                M::source_path(),
                M::ports(),
                config,
            )
            .whatever_context(
                "Failed to build or retrieve verilator dynamic library. Try removing the build directory if it is corrupted.",
            )?;

        let delete_model: extern "C" fn(*mut ffi::c_void) = *unsafe {
            library.get(format!("ffi_delete_V{}", M::name()).as_bytes())
        }
        .expect("failed to get symbol");

        let model = M::init_from(library, config.enable_tracing);

        self.model_deallocators.borrow_mut().push((
            // SAFETY: todo
            unsafe { model.model() },
            delete_model,
        ));

        Ok(model)
    }

    // TODO: should this be unified with the normal create_model by having
    // DynamicVerilatedModel implement VerilatedModel?

    /// Constructs a new dynamic model. Uses lazy and incremental building for
    /// efficiency. You must guarantee the correctness of the suppplied
    /// information, namely, that `name` is precisely the name of the
    /// Verilog module, `source_path` is, when canonicalized
    /// using [`fs::canonicalize`], the relative/absolute path to the Verilog
    /// file defining the module `name`, and `ports` is a correct subset of
    /// the ports of the Verilog module.
    ///
    /// ```no_run
    /// # use marlin_verilator::*;
    /// # use marlin_verilator::dynamic::*;
    /// # let runtime = VerilatorRuntime::new("".as_ref(), &[], &[], [], Default::default()).unwrap();
    /// # || -> Result<(), snafu::Whatever> {
    /// let mut main = runtime.create_dyn_model(
    ///    "main",
    ///    "src/main.sv",
    ///    &[
    ///        ("medium_input", 31, 0, PortDirection::Input),
    ///        ("medium_output", 31, 0, PortDirection::Output),
    ///    ],
    ///    VerilatedModelConfig::default(),
    /// )?;
    /// # Ok(()) };
    /// ````
    ///
    /// See also: [`VerilatorRuntime::create_model`]
    pub fn create_dyn_model<'ctx>(
        &'ctx self,
        name: &str,
        source_path: &str,
        ports: &[(&str, usize, usize, PortDirection)],
        config: VerilatedModelConfig,
    ) -> Result<DynamicVerilatedModel<'ctx>, Whatever> {
        let library = self
            .build_or_retrieve_library(name, source_path, ports, &config)
            .whatever_context(
                "Failed to build or retrieve verilator dynamic library. Try removing the build directory if it is corrupted.",
            )?;

        let new_main: extern "C" fn() -> *mut ffi::c_void =
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

        self.model_deallocators
            .borrow_mut()
            .push((main, delete_main));

        Ok(DynamicVerilatedModel {
            ports,
            name: name.to_string(),
            main,
            eval_main,
            library,
        })
    }

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
        config: &VerilatedModelConfig,
    ) -> Result<&Library, Whatever> {
        if name.chars().any(|c| c == '\\' || c == ' ') {
            whatever!("Escaped module names are not supported");
        }

        if self.options.log {
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
        if let Some((port, _, _, _)) =
            ports.iter().find(|(_, high, low, _)| high + 1 - low > 64)
        {
            whatever!(
                "Port {} on module {} is greater than 64 bits",
                port,
                name
            );
        }

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
                    "{name}_{}_{}",
                    source_path.replace("_", "__").replace("/", "_"),
                    library_key.hash
                );
                let local_artifacts_directory =
                    self.artifact_directory.join(&local_directory_name);

                if self.options.log {
                    log::info!(
                        "Creating artifacts directory {}",
                        local_artifacts_directory
                    );
                }
                fs::create_dir_all(&local_artifacts_directory)
                    .whatever_context(format!(
                        "Failed to create artifacts directory {}",
                        local_artifacts_directory,
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

                let _thread_lock = if let Ok(_thread_lock) =
                    thread_mutex.try_lock()
                {
                    //eprintln_nocapture!(
                    //    "thread-level try lock for {:?} succeeded",
                    //    std::thread::current().id()
                    //)?;
                    _thread_lock
                } else {
                    eprintln_nocapture!(
                        "{} waiting for file lock on build directory",
                        "    Blocking".bold().cyan(),
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
                if self.options.log {
                    log::info!("Acquiring file lock on artifact directory");
                }
                let lockfile = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(self.artifact_directory.join(format!("{local_directory_name}.lock")))
                .whatever_context(
                    "Failed to open lockfile for artifacts directory (this is not the actual lock itself, it is an I/O error)",
                )?;

                let _file_lock = file_guard::lock(
                    &lockfile,
                    file_guard::Lock::Exclusive,
                    0,
                    1,
                )
                .whatever_context(
                    "Failed to acquire file lock for artifacts directory",
                )?;
                //eprintln_nocapture!(
                //    "lockfile for {:?} succeeded",
                //    std::thread::current().id()
                //)?;

                let start = Instant::now();

                if self.options.log {
                    log::info!("Building the dynamic library with verilator");
                }
                let (library_path, was_rebuilt) = build_library(
                    &self.source_files,
                    &self.include_directories,
                    &self.dpi_functions,
                    name,
                    ports,
                    &local_artifacts_directory,
                    &self.options,
                    config,
                    self.options.log,
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
                .whatever_context(
                    "Failed to build verilator dynamic library",
                )?;

                if self.options.log {
                    log::info!("Opening the dynamic library");
                }
                let library = unsafe { Library::new(library_path) }
                    .whatever_context(
                        "Failed to load verilator dynamic library",
                    )?;

                one_time_library_setup(
                    &library,
                    &self.dpi_functions,
                    config.enable_tracing,
                    &self.options,
                )?;

                let library_idx = self.library_arena.push(library);
                entry.insert(library_idx);

                let end = Instant::now();
                let duration = end - start;

                if was_rebuilt {
                    eprintln_nocapture!(
                        "{} `verilator-{}` profile target in {}.{:02}s",
                        "    Finished".bold().green(),
                        if config.verilator_optimization == 0 {
                            "unoptimized".into()
                        } else {
                            format!("O{}", config.verilator_optimization)
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
