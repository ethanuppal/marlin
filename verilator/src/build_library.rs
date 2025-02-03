// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

// hardcoded knowledge:
// - output library is obj_dir/libV${top_module}.a
// - location of verilated.h
// - verilator library is obj_dir/libverilated.a

use std::{fmt::Write, fs, process::Command};

use camino::{Utf8Path, Utf8PathBuf};
use snafu::{prelude::*, Whatever};

use crate::PortDirection;

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

fn needs_rebuild(
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

pub fn build_library(
    source_files: &[&str],
    top_module: &str,
    ports: &[(&str, usize, usize, PortDirection)],
    artifact_directory: &Utf8Path,
) -> Result<Utf8PathBuf, Whatever> {
    let ffi_artifact_directory = artifact_directory.join("ffi");
    fs::create_dir_all(&ffi_artifact_directory).whatever_context(
        "Failed to create ffi subdirectory under artifacts directory",
    )?;
    let verilator_artifact_directory = artifact_directory.join("obj_dir");
    let library_name = format!("V{}_dyn", top_module);
    let library_path =
        verilator_artifact_directory.join(format!("lib{}.so", library_name));

    if !needs_rebuild(source_files, &verilator_artifact_directory)
        .whatever_context("Failed to check if artifacts need rebuilding")?
    {
        return Ok(library_path);
    }

    let _ffi_wrappers = build_ffi(&ffi_artifact_directory, top_module, ports)
        .whatever_context("Failed to build FFI wrappers")?;

    // bug in verilator#5226 means the directory must be relative to -Mdir
    let ffi_wrappers = Utf8Path::new("../ffi/ffi.cpp");

    let verilator_output = Command::new("verilator")
        .args(["--cc", "-sv", "--build", "-j", "0"])
        .args(["-CFLAGS", "-shared -fpic"])
        .args(["--lib-create", &library_name])
        .args(["--Mdir", verilator_artifact_directory.as_str()])
        .args(["--top-module", top_module])
        //.arg("-O3")
        .args(source_files)
        .arg(ffi_wrappers)
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
