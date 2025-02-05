// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::HashMap, env, fmt, path::PathBuf};

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use sv_parser::{self as sv, unwrap_node, Locate, RefNode};
use syn::{parse_macro_input, spanned::Spanned};
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

    let port_declarations_list = module
        .nodes
        .0
        .nodes
        .6
        .as_ref()
        .and_then(|list| list.nodes.0.nodes.1.as_ref())
        .map(|list| list.contents())
        .unwrap_or(vec![]);

    let mut ports = vec![];
    for (_, port) in port_declarations_list {
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
                            sv::DataTypeOrImplicit::DataType(data_type) => {
                                match &**data_type {
                                    sv::DataType::Vector(data_type_vector) => {
                                        &data_type_vector.nodes.2
                                    }
                                    other => todo!(
                                        "Unsupported data type {:?}",
                                        other
                                    ),
                                }
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

enum DPIPrimitiveType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

impl fmt::Display for DPIPrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DPIPrimitiveType::U8 => "u8",
            DPIPrimitiveType::U16 => "u16",
            DPIPrimitiveType::U32 => "u32",
            DPIPrimitiveType::U64 => "u64",
            DPIPrimitiveType::I8 => "i8",
            DPIPrimitiveType::I16 => "i16",
            DPIPrimitiveType::I32 => "i32",
            DPIPrimitiveType::I64 => "i64",
        }
        .fmt(f)
    }
}

impl DPIPrimitiveType {
    fn as_c(&self) -> &'static str {
        match self {
            // verilator uses signed for everything it seems
            DPIPrimitiveType::U8 => "int8_t",
            DPIPrimitiveType::U16 => "int16_t",
            DPIPrimitiveType::U32 => "int32_t",
            DPIPrimitiveType::U64 => "int64_t",
            DPIPrimitiveType::I8 => "int8_t",
            DPIPrimitiveType::I16 => "int16_t",
            DPIPrimitiveType::I32 => "int32_t",
            DPIPrimitiveType::I64 => "int64_t",
        }
    }
}

fn parse_dpi_primitive_type(
    ty: &syn::TypePath,
) -> Result<DPIPrimitiveType, syn::Error> {
    if let Some(qself) = &ty.qself {
        return Err(syn::Error::new_spanned(
            qself.lt_token,
            "Primitive integer type should not be qualified in DPI function",
        ));
    }

    match ty
        .path
        .require_ident()
        .or(Err(syn::Error::new_spanned(
            ty,
            "Primitive integer type should not have multiple path segments",
        )))?
        .to_string()
        .as_str()
    {
        "u8" => Ok(DPIPrimitiveType::U8),
        "u16" => Ok(DPIPrimitiveType::U16),
        "u32" => Ok(DPIPrimitiveType::U32),
        "u64" => Ok(DPIPrimitiveType::U64),
        "i8" => Ok(DPIPrimitiveType::I8),
        "i16" => Ok(DPIPrimitiveType::I16),
        "i32" => Ok(DPIPrimitiveType::I32),
        "i64" => Ok(DPIPrimitiveType::I64),
        _ => Err(syn::Error::new_spanned(
            ty,
            "Unknown primitive integer type",
        )),
    }
}

enum DPIType {
    Input(DPIPrimitiveType),
    Output(DPIPrimitiveType),
    Inout(DPIPrimitiveType),
}

fn parse_dpi_type(
    direction: (PortDirection, &syn::Attribute),
    ty: &syn::Type,
) -> Result<DPIType, syn::Error> {
    match ty {
        syn::Type::Path(type_path) => {
            if matches!(direction.0, PortDirection::Output | PortDirection::Inout) {
                return Err(syn::Error::new_spanned(direction.1, "DPI output or inout type must use a lifetime-free mutable reference"));
            }
            Ok(DPIType::Input(parse_dpi_primitive_type(type_path)?))
        },
        syn::Type::Reference(syn::TypeReference {
            and_token,
            lifetime,
            mutability,
            elem,
        }) => {
            if matches!(direction.0, PortDirection::Input) {
                return Err(syn::Error::new_spanned(direction.1, "DPI input type must use a bare primitive integer"));
            }
            if mutability.is_none() {
                return Err(syn::Error::new_spanned(and_token, "DPI output or inout type must be represented with a mutable reference"));
            }
           if let Some(lifetime) = lifetime {
                return Err(syn::Error::new_spanned(lifetime, "DPI output or inout type cannot use lifetimes"));
            }

            let syn::Type::Path(type_path) = elem.as_ref() else {

                return Err(syn::Error::new_spanned(elem, "DPI output or inout type must be a mutable reference to a primitive integer type"));
};
            let inner = parse_dpi_primitive_type(type_path)?;
match direction.0 {
                PortDirection::Output => Ok(DPIType::Output(inner)),
                PortDirection::Inout => Ok(DPIType::Inout(inner)),
                _ => unreachable!()
            }
        },
        other => Err(syn::Error::new_spanned(other, "This type is not supported in DPI. Please use primitive integers or mutable references to them")),
    }
}

#[proc_macro_attribute]
pub fn dpi(_args: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as syn::ItemFn);

    let Some(abi) = &item_fn.sig.abi else {
        return syn::Error::new_spanned(
            item_fn,
            "`extern \"C\"` is required to expose this Rust function to C",
        )
        .into_compile_error()
        .into();
    };

    if !abi
        .name
        .as_ref()
        .map(|name| name.value().as_str() == "C")
        .unwrap_or(true)
    {
        return syn::Error::new_spanned(
            item_fn,
            "You must specify the C ABI for the `extern` marking",
        )
        .into_compile_error()
        .into();
    }

    if !item_fn.attrs.iter().any(|attribute| {
        attribute
            .path()
            .segments
            .first()
            .map(|segment| segment.ident.to_string().as_str() == "no_mangle")
            .unwrap_or(false)
    }) {
        return syn::Error::new_spanned(
            item_fn,
            "`#[no_mangle]` is required to expose this Rust function to C",
        )
        .into_compile_error()
        .into();
    }

    if item_fn.sig.generics.lt_token.is_some() {
        return syn::Error::new_spanned(
            item_fn.sig.generics,
            "Generics are not supported for DPI functions",
        )
        .into_compile_error()
        .into();
    }

    if let Some(asyncness) = &item_fn.sig.asyncness {
        return syn::Error::new_spanned(
            asyncness,
            "DPI functions must be synchronous",
        )
        .into_compile_error()
        .into();
    }

    if let syn::ReturnType::Type(_, return_type) = &item_fn.sig.output {
        return syn::Error::new_spanned(
            return_type,
            "DPI functions cannot have a return value",
        )
        .into_compile_error()
        .into();
    }

    let ports = match item_fn.sig.inputs.iter().try_fold(vec![], |mut ports, input| {
        let syn::FnArg::Typed(parameter) = input else {
            return Err(syn::Error::new_spanned(input, "Invalid parameter on DPI function"));
        };

        let syn::Pat::Ident(name) = &*parameter.pat else {
            return Err(syn::Error::new_spanned(parameter, "Function argument must be an identifier"));
    };

        let Some(direction)  = parameter.attrs.iter().find_map(|attr| {
            match attr.path().require_ident().ok()?.to_string().as_str() {
                "input" => Some((PortDirection::Input, attr)),
                "output" => Some((PortDirection::Output, attr)),
                "inout" => Some((PortDirection::Inout, attr)),
                _ => None
            }
        }) else {
            return Err(syn::Error::new_spanned(parameter, "Specify `#[input]`, `#[output]`, or `#[inout]` on the parameter"));
        };

        let attrs = parameter.attrs.iter().filter(|&attribute| {
            attribute.path().require_ident().ok().map(|ident|
                !matches!(ident.to_string().as_str(), "input" | "output" | "inout")).unwrap_or(false)
        }).cloned().collect::<Vec<_>>();

        ports.push((name, attrs, parse_dpi_type(direction, &parameter.ty)?));
        Ok(ports)
    }) {
        Ok(ports) => ports,
        Err(error) => {
            return error.into_compile_error().into();
        }
    };

    let mut cloned_item_fn = item_fn.clone();
    for input in &mut cloned_item_fn.sig.inputs {
        if let syn::FnArg::Typed(parameter) = input {
            parameter.attrs.retain(|attribute| {
                !attribute
                    .path()
                    .require_ident()
                    .ok()
                    .map(|ident| {
                        matches!(
                            ident.to_string().as_str(),
                            "input" | "output" | "inout"
                        )
                    })
                    .unwrap_or(false)
            });
        }
    }

    let attributes = item_fn.attrs;
    let name = item_fn.sig.ident;
    let body = item_fn.block;

    let parameters = ports.iter().map(|(name, attributes, dpi_type)| {
        let parameter_type = match dpi_type {
            DPIType::Input(inner) => {
                let type_ident = format_ident!("{}", inner.to_string());
                quote! { #type_ident }
            }
            DPIType::Output(inner) | DPIType::Inout(inner) => {
                let type_ident = format_ident!("{}", inner.to_string());
                quote! { *mut #type_ident }
            }
        };
        quote! {
            #(#attributes)* #name: #parameter_type
        }
    });

    let preamble =
        ports
            .iter()
            .filter_map(|(name, _, dpi_type)| match dpi_type {
                DPIType::Output(_) | DPIType::Inout(_) => Some(quote! {
                    let #name = unsafe { &mut *#name };
                }),
                _ => None,
            });

    let function_name = format_ident!("rust_{}", name);
    let function_to_compile = quote! {
        #(#attributes)*
        pub extern "C" fn #function_name(#(#parameters),*) {
            #(#preamble)*
            #body
        }
    }
    .to_string();

    let name_literal = syn::LitStr::new(name.to_string().as_str(), name.span());
    let function_to_compile_literal =
        syn::LitStr::new(&function_to_compile, cloned_item_fn.span());

    let c_signature = ports
        .iter()
        .map(|(name, _, dpi_type)| {
            let c_type = match dpi_type {
                DPIType::Input(inner) => inner.as_c().to_string(),
                DPIType::Output(inner) | DPIType::Inout(inner) => {
                    format!("{}*", inner.as_c())
                }
            };
            format!("{} {}", c_type, name.ident)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let c_arguments = ports
        .iter()
        .map(|(name, _, _)| name.ident.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let c_function = format!(
        "extern \"C\" void rust_{name}({c_signature});\nextern \"C\" void {name}({c_signature}) {{ rust_{name}({c_arguments}); }}",
    );
    let c_function_literal =
        syn::LitStr::new(&c_function, cloned_item_fn.span());
    quote! {
        #[allow(non_upper_case_globals)]
        const #name: verilog::__reexports::verilator::DpiFunction = {
            #cloned_item_fn
            verilog::__reexports::verilator::DpiFunction(
                #name_literal,
                #c_function_literal,
                #function_to_compile_literal
            )
        };
    }
    .into()
}
