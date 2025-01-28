// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::HashMap, env, path::PathBuf};

use proc_macro::TokenStream;
use sv_parser::{self as sv, unwrap_node, Locate, RefNode};
use verilator::PortDirection;
use verilog_macro_builder::{build_verilated_struct, MacroArgs};

mod util;

#[proc_macro_attribute]
pub fn verilog(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as MacroArgs);

    let manifest_directory = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Please compile using `cargo` or set the `CARGO_MANIFEST_DIR` environment variable"));
    let source_path = manifest_directory.join(args.source_path.value());

    let defines = HashMap::new();
    let (ast, _) =
        match sv::parse_sv(&source_path, &defines, &["."], false, false) {
            Ok(result) => result,
            Err(error) => {
                return syn::Error::new_spanned(
                    args.source_path,
                    error.to_string(),
                )
                .into_compile_error()
                .into();
            }
        };

    let Some(module) = (&ast).into_iter().find_map(|node| match node {
        RefNode::ModuleDeclarationAnsi(module) => {
            // taken from https://github.com/dalance/sv-parser/blob/master/README.md
            fn get_identifier(node: RefNode) -> Option<Locate> {
                match unwrap_node!(node, SimpleIdentifier, EscapedIdentifier) {
                    Some(RefNode::SimpleIdentifier(x)) => Some(x.nodes.0),
                    Some(RefNode::EscapedIdentifier(x)) => Some(x.nodes.0),
                    _ => None,
                }
            }

            let id = unwrap_node!(module, ModuleIdentifier).unwrap();
            let id = get_identifier(id).unwrap();
            let id = ast.get_str_trim(&id).unwrap();
            if id == args.name.value().as_str() {
                Some(module)
            } else {
                None
            }
        }
        _ => None,
    }) else {
        return syn::Error::new_spanned(
            &args.name,
            format!(
                "Could not find module declaration for `{}` in {}",
                args.name.value(),
                args.source_path.value()
            ),
        )
        .into_compile_error()
        .into();
    };

    let Some(port_declarations_list) = module
        .nodes
        .0
        .nodes
        .6
        .as_ref()
        .and_then(|list| list.nodes.0.nodes.1.as_ref())
    else {
        return syn::Error::new_spanned(
            &args.name,
            format!(
                "Module `{}` is missing a list of ports",
                args.name.value()
            ),
        )
        .into_compile_error()
        .into();
    };

    let mut ports = vec![];
    for (_, port) in port_declarations_list.contents() {
        match port {
            sv::AnsiPortDeclaration::Net(net) => {
                let port_name = ast.get_str_trim(&net.nodes.1.nodes.0).expect(
                    "Port identifier could not be traced back to source code",
                );

                if port_name.chars().any(|c| c == '\\' || c == ' ') {
                    return syn::Error::new_spanned(
                        args.name,
                        "Escaped module names are not supported",
                    )
                    .into_compile_error()
                    .into();
                }

                let Some((port_direction, port_type ))= net.nodes.0.as_ref().and_then(|maybe_net_header| match maybe_net_header {
                    sv::NetPortHeaderOrInterfacePortHeader::NetPortHeader(net_port_header) => {
                        net_port_header.nodes.0.as_ref().map(|port_direction| (port_direction, &net_port_header.nodes.1))
                    },
                    _ => todo!("Other port header")
                }) else {
                    return syn::Error::new_spanned(
                        args.source_path,
                        format!(
                            "Port `{}` has no supported direction (`input` or `output`)",
                            port_name
                        ),
                    )
                    .into_compile_error().into();
                };

                let port_dimensions = match port_type {
                    sv::NetPortType::DataType(net_port_type_data_type) => {
                        match &net_port_type_data_type.nodes.1 {
                            sv::DataTypeOrImplicit::DataType(_data_type) => {
                                todo!("a")
                            }
                            sv::DataTypeOrImplicit::ImplicitDataType(
                                implicit_data_type,
                            ) => &implicit_data_type.nodes.1,
                        }
                    }
                    sv::NetPortType::NetTypeIdentifier(
                        _net_type_identifier,
                    ) => todo!("bklk"),
                    sv::NetPortType::Interconnect(
                        _net_port_type_interconnect,
                    ) => todo!("ckl"),
                };

                let (port_msb, port_lsb) = match port_dimensions.len() {
                    0 => (0, 0),
                    1 => match &port_dimensions[0] {
                        sv::PackedDimension::Range(packed_dimension_range) => {
                            let range =
                                &packed_dimension_range.nodes.0.nodes.1.nodes;
                            (
                                util::evaluate_numeric_constant_expression(
                                    &ast, &range.0,
                                ),
                                util::evaluate_numeric_constant_expression(
                                    &ast, &range.2,
                                ),
                            )
                        }
                        _ => todo!(),
                    },
                    _ => todo!("Don't support multidimensional ports yet"),
                };

                let port_direction = match port_direction {
                    sv::PortDirection::Input(_) => PortDirection::Input,
                    sv::PortDirection::Output(_) => PortDirection::Output,
                    sv::PortDirection::Inout(_) => PortDirection::Inout,
                    sv::PortDirection::Ref(_) => todo!(),
                };

                ports.push((
                    port_name.to_string(),
                    port_msb,
                    port_lsb,
                    port_direction,
                ));
            }
            _ => todo!("Other types of ports"),
        }
    }

    build_verilated_struct(
        "verilog",
        args.name,
        syn::LitStr::new(
            source_path.to_string_lossy().as_ref(),
            args.source_path.span(),
        ),
        ports,
        args.clock_port,
        args.reset_port,
        item.into(),
    )
    .into()
}
