// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{env, fs};

use camino::Utf8PathBuf;
use proc_macro::TokenStream;
use spade_parser::Logos;
use verilator::PortDirection;
use verilog_macro_builder::{build_verilated_struct, MacroArgs};

fn search_for_swim_toml(mut start: Utf8PathBuf) -> Option<Utf8PathBuf> {
    while !start.as_str().is_empty() {
        if start.join("swim.toml").is_file() {
            return Some(start.join("swim.toml"));
        }
        start.pop();
    }
    None
}

#[proc_macro_attribute]
pub fn spade(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as MacroArgs);

    let manifest_directory = Utf8PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("Please use CARGO"),
    );
    let Some(swim_toml) = search_for_swim_toml(manifest_directory) else {
        return syn::Error::new_spanned(
            args.source_path,
            "Could not find swim.toml",
        )
        .into_compile_error()
        .into();
    };

    let verilog_source_path = {
        let mut source_path = swim_toml.clone();
        source_path.pop();
        source_path.push("build/spade.sv");
        syn::LitStr::new(source_path.as_str(), args.source_path.span())
    };

    let spade_source_path = {
        let mut spade_source_path = swim_toml.clone();
        spade_source_path.pop();
        spade_source_path.join(args.source_path.value())
    };
    let source_code = match fs::read_to_string(&spade_source_path) {
        Ok(contents) => contents,
        Err(error) => {
            return syn::Error::new_spanned(
                &args.source_path,
                format!(
                    "Failed to read source code file at {}: {}",
                    spade_source_path, error
                ),
            )
            .into_compile_error()
            .into();
        }
    };

    let lexer = <spade_parser::lexer::TokenKind as Logos>::lexer(&source_code);
    let mut parser = spade_parser::Parser::new(lexer, 0);
    let top_level = match parser.top_level_module_body() {
        Ok(body) => body,
        Err(_error) => {
            return syn::Error::new_spanned(
                args.source_path,
                "Failed to parse Spade code: run the Spade compiler for more details",
            )
            .into_compile_error()
            .into();
        }
    };

    let Some(unit_head) =
        top_level.members.into_iter().find_map(|item| match item {
            spade_ast::Item::Unit(unit)
                if unit.head.name.0.as_str() == args.name.value().as_str() =>
            {
                Some(unit.head.clone()) // why clone?
            }
            _ => None,
        })
    else {
        return syn::Error::new_spanned(
            &args.name,
            format!(
                "Could not find top-level unit named `{}` in {}. Remember to use `#[no_mangle]`",
                args.name.value(),
                args.source_path.value()
            ),
        )
        .into_compile_error()
        .into();
    };

    if !unit_head
        .attributes
        .0
        .iter()
        .any(|attribute| attribute.name() == "no_mangle")
    {
        return syn::Error::new_spanned(
            &args.name,
            format!("Annotate `{}` with `#[no_mangle]`", args.name.value()),
        )
        .into_compile_error()
        .into();
    }

    if unit_head.output_type.is_some() {
        return syn::Error::new_spanned(
            &args.name,
            format!(
                "Unsupported output type on `{}` (verilator makes this annoying): use `inv &` instead",
                args.name.value()
            ),
        )
        .into_compile_error()
        .into();
    }

    let mut ports = vec![];
    for (attributes, port_name, port_type) in &unit_head.inputs.inner.args {
        if !attributes
            .0
            .iter()
            .any(|attribute| attribute.name() == "no_mangle")
        {
            return syn::Error::new_spanned(
                &args.name,
                format!(
                    "Annotate the port `{}` on unit `{}` with `#[no_mangle]`",
                    port_name.inner,
                    args.name.value()
                ),
            )
            .into_compile_error()
            .into();
        }

        let port_direction = match &port_type.inner {
            spade_ast::TypeSpec::Inverted(_) => PortDirection::Output,
            _ => PortDirection::Input,
        };

        let port_msb = spade_simple_type_width(&port_type.inner) - 1;

        ports.push((port_name.inner.0.clone(), port_msb, 0, port_direction));
    }

    build_verilated_struct(
        "spade",
        args.name,
        verilog_source_path,
        ports,
        args.clock_port,
        args.reset_port,
        item.into(),
    )
    .into()
}

// TODO: make this decent with error handling. this is some of the worst code
// I've written. This implementation is based off of https://gitlab.com/spade-lang/spade/-/blob/79cfd7ed12ee8a7328aa6e6650e394ed55ed2b2c/spade-mir/src/types.rs
/// Determines the bit-width of a "simple" type present in a Spade top exposed
/// to Verilog, e.g., integers and inverted integers, clocks, etc.
fn spade_simple_type_width(type_spec: &spade_ast::TypeSpec) -> usize {
    fn get_type_spec(
        type_expression: &spade_ast::TypeExpression,
    ) -> &spade_ast::TypeSpec {
        match type_expression {
            spade_ast::TypeExpression::TypeSpec(type_spec) => type_spec,
            _ => panic!("Expected a type spec"),
        }
    }

    fn get_constant(type_expression: &spade_ast::TypeExpression) -> usize {
        // TODO: handle bigints correctly
        match type_expression {
            spade_ast::TypeExpression::Integer(big_int) => {
                big_int.to_u64_digits().1[0] as usize
            }
            _ => panic!("Expected an integer"),
        }
    }

    match type_spec {
        spade_ast::TypeSpec::Tuple(inner) => inner
            .iter()
            .map(|type_expression| {
                spade_simple_type_width(get_type_spec(type_expression))
            })
            .sum(),
        spade_ast::TypeSpec::Named(name, args) => {
            if name.inner.0.len() != 1 {
                panic!("I'm so done writing error messages");
            }
            match name.inner.0[0].inner.0.as_str() {
                "int" | "uint" => {
                    if args.is_none() {
                        panic!("I don't want to write error messages");
                    }
                    if args.as_ref().unwrap().len() != 1 {
                        panic!("I don't want to write error messages");
                    }
                    get_constant(&args.as_ref().unwrap().inner[0])
                }
                _ => panic!("I DONT WANT TO WRITE ERROR MESSAGES"),
            }
        }
        spade_ast::TypeSpec::Array { inner, size } => {
            spade_simple_type_width(get_type_spec(inner)) * get_constant(size)
        }
        spade_ast::TypeSpec::Inverted(inner) => {
            spade_simple_type_width(get_type_spec(inner))
        }
        spade_ast::TypeSpec::Wire(inner) => {
            spade_simple_type_width(get_type_spec(inner))
        }
        spade_ast::TypeSpec::Unit(_) | spade_ast::TypeSpec::Wildcard => {
            panic!("Invalid type for Verilog-exposed Spade top")
        }
    }
}
