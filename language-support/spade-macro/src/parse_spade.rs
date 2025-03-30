// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::HashMap, fs::File, io::Read};

use camino::Utf8Path;
use snafu::{whatever, ResultExt, Whatever};
use spade::ModuleNamespace;
use spade_codespan_reporting::term::termcolor::Buffer;
use spade_common::{
    location_info::Loc,
    name::{NameID, Path},
};
use spade_diagnostics::{emitter::CodespanEmitter, DiagHandler};
use spade_hir::TypeDeclaration;
use swim::spade::SpadeFile;

pub fn parse_spade(
    root_dir: &Utf8Path,
    config: &swim::config::Config,
) -> Result<HashMap<NameID, Loc<TypeDeclaration>>, Whatever> {
    let mut buffer = Buffer::no_color();
    let infiles = swim::spade::collect_namespaced_spade_files(root_dir, config)
        .whatever_context("Failed to find Spade files")?;

    let sources: Result<Vec<(ModuleNamespace, String, String)>, Whatever> =
        infiles
            .into_iter()
            .map(|SpadeFile { namespace, path }| {
                let mut file =
                    File::open(&path).with_whatever_context(|_| {
                        format!("Failed to open {}", path)
                    })?;
                let mut file_content = String::new();
                file.read_to_string(&mut file_content).whatever_context(
                    format!("Failed to read Spade file {path}"),
                )?;
                Ok((
                    ModuleNamespace {
                        namespace: Path::from_strs(&[&namespace.namespace]),
                        base_namespace: Path::from_strs(&[
                            &namespace.base_namespace
                        ]),
                        file: path.to_string(),
                    },
                    path.to_string(),
                    file_content,
                ))
            })
            .collect();

    let opts = spade::Opt {
        error_buffer: &mut buffer,
        outfile: None,
        mir_output: None,
        state_dump_file: None,
        item_list_file: None,
        print_type_traceback: false,
        print_parse_traceback: false,
        verilator_wrapper_output: None,
        opt_passes: vec![],
    };

    let diag_handler = DiagHandler::new(Box::new(CodespanEmitter));
    let artifacts = spade::compile(sources.unwrap(), true, opts, diag_handler)
        .or_else(|_| whatever!("{buffer:?}"))?;

    Ok(artifacts.item_list.types)
}
