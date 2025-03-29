// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::fs::File;

use snafu::Whatever;
use spade::{namespaced_file::NamespacedFile, ModuleNamespace};
use spade_codespan_reporting::term::termcolor::Buffer;
use spade_diagnostics::{emitter::CodespanEmitter, DiagHandler};

use crate::swim::SpadeFile;

fn parse_spade() -> Result<Vec<SpadeFile>, Whatever> {
    let mut buffer = Buffer::no_color();
    let infiles = todo!();

    let sources: Result<Vec<(ModuleNamespace, String, String)>, Whatever> =
        infiles
            .into_iter()
            .map(
                |NamespacedFile {
                     file: infile,
                     namespace,
                     base_namespace,
                 }| {
                    let mut file =
                        File::open(&infile).with_whatever_context(|_| {
                            format!(
                                "Failed to open {}",
                                &infile.to_string_lossy()
                            )
                        })?;
                    let mut file_content = String::new();
                    file.read_to_string(&mut file_content)?;
                    Ok((
                        ModuleNamespace {
                            namespace,
                            base_namespace,
                            file: infile.to_string_lossy().to_string(),
                        },
                        infile.to_string_lossy().to_string(),
                        file_content,
                    ))
                },
            )
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
    // Codespan emitter so compilation errors are reported as normal.
    let diag_handler = DiagHandler::new(Box::new(CodespanEmitter));
    let artefacts = spade::compile(sources.unwrap(), true, opts, diag_handler)
        .map_err(|_| buffer)?;
}
