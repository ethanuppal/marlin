// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{env, fs};

use camino::Utf8PathBuf;
use marlin_verilog_macro_builder::{
    build_verilated_struct, parse_verilog_ports, MacroArgs,
};
use proc_macro::TokenStream;

fn search_for_veryl_toml(mut start: Utf8PathBuf) -> Option<Utf8PathBuf> {
    while !start.as_str().is_empty() {
        if start.join("Veryl.toml").is_file() {
            return Some(start.join("Veryl.toml"));
        }
        start.pop();
    }
    None
}

#[proc_macro_attribute]
pub fn veryl(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as MacroArgs);

    let manifest_directory = Utf8PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("Please use CARGO"),
    );
    let Some(veryl_toml) = search_for_veryl_toml(manifest_directory) else {
        return syn::Error::new_spanned(
            args.source_path,
            "Could not find Veryl.toml",
        )
        .into_compile_error()
        .into();
    };

    let veryl_source_path = {
        let mut veryl_source_path = veryl_toml.clone();
        veryl_source_path.pop();
        veryl_source_path.join(args.source_path.value())
    };
    let source_code = match fs::read_to_string(&veryl_source_path) {
        Ok(contents) => contents,
        Err(error) => {
            return syn::Error::new_spanned(
                &args.source_path,
                format!(
                    "Failed to read source code file at {}: {}",
                    veryl_source_path, error
                ),
            )
            .into_compile_error()
            .into();
        }
    };

    let parser = Parser::new();

    let verilog_source_path = syn::LitStr::new(
        veryl_source_path.with_extension("sv").as_str(),
        args.source_path.span(),
    );

    let verilog_module_prefix = veryl_source_path
        .file_stem()
        .map(|stem| format!("{}_", stem))
        .unwrap_or_default();

    let verilog_module_name = syn::LitStr::new(
        &format!("{}{}", verilog_module_prefix, args.name.value()),
        args.name.span(),
    );

    let ports = match parse_verilog_ports(
        &args.name,
        &args.source_path,
        verilog_source_path.value().as_ref(),
    ) {
        Ok(ports) => ports,
        Err(error) => {
            return error.into();
        }
    };

    build_verilated_struct(
        "veryl",
        verilog_module_name,
        verilog_source_path,
        ports,
        args.clock_port,
        args.reset_port,
        item.into(),
    )
    .into()
}
