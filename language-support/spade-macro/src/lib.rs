// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::HashMap, env, fs};

use camino::Utf8PathBuf;
use marlin_verilator::PortDirection;
use marlin_verilog_macro_builder::{build_verilated_struct, MacroArgs};
use parse_spade::parse_spade;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use spade_hir::TypeDeclaration;
use spade_parser::logos::Logos;
use swim::config::Config;

mod parse_spade;

fn search_for_swim_toml(mut start: Utf8PathBuf) -> Option<Utf8PathBuf> {
    while start.parent().is_some() {
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
        top_level.members.iter().find_map(|item| match item {
            spade_ast::Item::Unit(unit)
                if unit.head.name.0.as_str() == args.name.value().as_str() =>
            {
                Some(unit.head.clone())
            }
            _ => None,
        })
    else {
        let names = top_level
            .members
            .iter()
            .filter_map(|item| match item {
                spade_ast::Item::Unit(unit) => Some(format!(
                    "{} {}",
                    unit.head.unit_kind, unit.head.name.0
                )),
                _ => None,
            })
            .collect::<Vec<_>>();
        return syn::Error::new_spanned(
            &args.name,
            format!(
                "Could not find top-level unit named `{}` in {}. Remember to use `#[no_mangle(all)]`. Unit names found are: {} (there are {} module item(s) in total)",
                args.name.value(),
                args.source_path.value(),
                if names.is_empty() {"<none found>".into()} else {names.join(", ")},
                top_level.members.len()
            ),
        )
        .into_compile_error()
        .into();
    };

    let Some(unit_mangle_attribute) = unit_head
        .attributes
        .0
        .iter()
        .find(|attribute| attribute.name() == "no_mangle")
    else {
        return syn::Error::new_spanned(
            &args.name,
            format!(
                "Annotate `{}` with `#[no_mangle(all)]`",
                args.name.value()
            ),
        )
        .into_compile_error()
        .into();
    };
    let is_no_mangle_all = matches!(
        unit_mangle_attribute.inner,
        spade_ast::Attribute::NoMangle { all: true }
    );

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
            && !is_no_mangle_all
        {
            return syn::Error::new_spanned(
                &args.name,
                format!(
                    "Annotate the unit `{}` with `#[no_mangle(all)]` or just the port `{}` with `#[no_mangle]`",
                    args.name.value(),
                    port_name.inner,
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
        spade_ast::TypeSpec::Wildcard => {
            panic!("Invalid type for Verilog-exposed Spade top")
        }
    }
}

#[proc_macro]
pub fn spade_types(_input: TokenStream) -> TokenStream {
    let manifest_directory = Utf8PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("Please use CARGO"),
    );
    let Some(swim_toml_path) = search_for_swim_toml(manifest_directory) else {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "Could not find swim.toml",
        )
        .into_compile_error()
        .into();
    };

    let root = {
        let mut root = swim_toml_path.clone();
        root.pop();
        root
    };

    let swim_toml_contents = match fs::read_to_string(&swim_toml_path) {
        Ok(contents) => contents,
        Err(error) => {
            return syn::Error::new(proc_macro2::Span::call_site(), format!("Could not read contents of swim.toml at project root {}: {}", swim_toml_path, error)).into_compile_error().into();
        }
    };
    let config: Config = match toml::from_str(&swim_toml_contents) {
        Ok(toml) => toml,
        Err(error) => {
            return syn::Error::new(proc_macro2::Span::call_site(), format!("Could not parse contents of Veryl.toml at project root {} as a TOML file: {}", swim_toml_path, error)).into_compile_error().into();
        }
    };

    let types = match parse_spade(&root, &config) {
        Ok(types) => types,
        Err(error) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "Failed to parse Spade files and determine types: {:?}",
                    error
                ),
            )
            .into_compile_error()
            .into();
        }
    };

    struct Module {
        types: Vec<TypeDeclaration>,
        children: HashMap<String, usize>,
    }

    let mut tree = vec![Module {
        types: vec![],
        children: HashMap::new(),
    }];

    for (name_id, type_declaration) in types {
        let path = name_id.1.prelude().as_strings();
        let mut current = 0;
        for part in path {
            current = if let Some(&child) = tree[current].children.get(&part) {
                child
            } else {
                let new_index = tree.len();
                tree.push(Module {
                    types: vec![],
                    children: HashMap::new(),
                });
                tree[current].children.insert(part, new_index);
                new_index
            };
        }
        tree[current].types.push(type_declaration.inner);
    }

    fn render(
        index: usize,
        tree: &[Module],
        depth: usize,
    ) -> proc_macro2::TokenStream {
        let module = &tree[index];
        let mut tokens = proc_macro2::TokenStream::new();
        for type_decl in &module.types {
            tokens.extend(spade_type_to_tokens(type_decl, depth));
        }
        for (child_name, &child_index) in &module.children {
            let child_tokens = render(child_index, tree, depth + 1);
            let ident =
                syn::Ident::new(child_name, proc_macro2::Span::call_site());
            tokens.extend(quote! {
                pub mod #ident {
                    use super::{ReadFromPorts, PinToPorts};

                    #child_tokens
                }
            });
        }
        tokens
    }

    render(0, &tree, 0).into()
}

fn spade_type_to_tokens(
    type_declaration: &TypeDeclaration,
    module_nesting: usize,
) -> proc_macro2::TokenStream {
    let name = format_ident!("{}", type_declaration.name.1.tail().0);

    let generic_arguments_option = if type_declaration.generic_args.is_empty() {
        quote! {}
    } else {
        let generic_arguments =
            type_declaration
                .generic_args
                .iter()
                .map(|generic_argument| {
                    let name = format_ident!(
                        "{}",
                        generic_argument.name_id.1.tail().0
                    );
                    match generic_argument.meta {
                        spade_types::meta_types::MetaType::Any => todo!(),
                        spade_types::meta_types::MetaType::Type => {
                            quote! { #name: ReadFromPorts + PinToPorts }
                        }
                        spade_types::meta_types::MetaType::Number => {
                            quote! { const #name: _  }
                        }
                        spade_types::meta_types::MetaType::Int => {
                            quote! { const #name: isize  }
                        }
                        spade_types::meta_types::MetaType::Uint => {
                            quote! { const #name: usize  }
                        }
                        spade_types::meta_types::MetaType::Bool => {
                            quote! { const #name: bool  }
                        }
                    }
                });
        quote! { <#(#generic_arguments),*> }
    };

    match &type_declaration.kind {
        spade_hir::TypeDeclKind::Enum(enum_declaration) => {
            let docs = syn::LitStr::new(
                &enum_declaration.documentation,
                proc_macro2::Span::call_site(),
            );
            let variants = enum_declaration.options.iter().enumerate().map(
                |(i, (variant_name, parameters))| {
                    let fields = if parameters.0.is_empty() {
                        syn::Fields::Unit
                    } else {
                        let mut named_fields =
                            syn::punctuated::Punctuated::new();
                        for parameter in &parameters.0 {
                            named_fields.push(syn::Field {
                                attrs: vec![],
                                vis: syn::Visibility::Inherited,
                                mutability: syn::FieldMutability::None,
                                ident: Some(format_ident!(
                                    "{}",
                                    parameter.name.0
                                )),
                                colon_token: Default::default(),
                                ty: spade_type_to_syn_type(
                                    &parameter.ty,
                                    module_nesting,
                                ),
                            })
                        }
                        syn::Fields::Named(syn::FieldsNamed {
                            brace_token: Default::default(),
                            named: named_fields,
                        })
                    };
                    syn::Variant {
                        attrs: if i == 0 {
                            vec![syn::parse_quote! { #[default] }]
                        } else {
                            vec![]
                        },
                        ident: format_ident!("{}", variant_name.1.tail().0),
                        fields,
                        discriminant: None,
                    }
                },
            );
            quote! {
                #[derive(Default)]
                #[doc = #docs]
                pub enum #name #generic_arguments_option {
                    #(#variants),*
                }
            }
        }
        spade_hir::TypeDeclKind::Primitive(primitive_type) => {
            match primitive_type {
                spade_types::PrimitiveType::Int => {
                    //type_declaration.generic_args[0]
                    //spade_simple_type_width(type_spec)
                    quote! {}
                }
                spade_types::PrimitiveType::Uint => quote! {},
                spade_types::PrimitiveType::Clock => quote! {},
                spade_types::PrimitiveType::Bool => quote! {},
                spade_types::PrimitiveType::Bit => quote! {},
                spade_types::PrimitiveType::Memory => quote! {},
                spade_types::PrimitiveType::InOut => quote! {},
            }
        }
        spade_hir::TypeDeclKind::Struct(struct_declaration) => quote! {},
    }
}

fn spade_type_to_syn_type(
    type_spec: &spade_hir::TypeSpec,
    module_nesting: usize,
) -> syn::Type {
    match type_spec {
        spade_hir::TypeSpec::Declared(name, params) => {
            if params.is_empty() {
                let mut segments = syn::punctuated::Punctuated::new();
                for segment in name.1.as_strings() {
                    segments.push(syn::PathSegment {
                        ident: format_ident!("{}", segment),
                        arguments: syn::PathArguments::None,
                    });
                }
                for i in 0..module_nesting {
                    segments.get_mut(i).unwrap().ident = format_ident!("super");
                }
                let path = syn::Path {
                    leading_colon: None,
                    segments,
                };
                syn::Type::Path(syn::TypePath { qself: None, path })
            } else {
                let generic_arguments = {
                    let mut generic_arguments =
                        syn::punctuated::Punctuated::new();
                    for param in params {
                        let ty: syn::Type = syn::parse_str(&param.to_string())
                            .unwrap_or_else(|_| syn::parse_quote!(_));
                        generic_arguments
                            .push_value(syn::GenericArgument::Type(ty));
                    }
                    syn::PathArguments::AngleBracketed(
                        syn::AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: Default::default(),
                            args: generic_arguments,
                            gt_token: Default::default(),
                        },
                    )
                };
                let mut segments = syn::punctuated::Punctuated::new();
                for segment in name.1.as_strings() {
                    segments.push(syn::PathSegment {
                        ident: format_ident!("{}", segment),
                        arguments: syn::PathArguments::None,
                    });
                }
                for i in 0..module_nesting {
                    segments.get_mut(i).unwrap().ident = format_ident!("super");
                }
                segments.last_mut().unwrap().arguments = generic_arguments;
                let path = syn::Path {
                    leading_colon: None,
                    segments,
                };
                syn::Type::Path(syn::TypePath { qself: None, path })
            }
        }
        spade_hir::TypeSpec::Generic(name) => {
            let mut segments = syn::punctuated::Punctuated::new();
            segments.push(syn::PathSegment {
                ident: format_ident!("{}", name.1.tail().0),
                arguments: syn::PathArguments::None,
            });
            let path = syn::Path {
                leading_colon: None,
                segments,
            };
            syn::Type::Path(syn::TypePath { qself: None, path })
        }
        spade_hir::TypeSpec::Tuple(members) => {
            let types = members
                .iter()
                .map(|member| spade_type_to_syn_type(member, module_nesting))
                .collect::<Vec<_>>();
            let elements = syn::punctuated::Punctuated::from_iter(types);
            syn::Type::Tuple(syn::TypeTuple {
                paren_token: Default::default(),
                elems: elements,
            })
        }
        spade_hir::TypeSpec::Array { inner, size } => {
            let inner_ty = spade_type_to_syn_type(inner, module_nesting);
            let size_expr: syn::Expr = syn::parse_str(&size.to_string())
                .unwrap_or_else(|_| syn::parse_quote!(0));
            syn::parse_quote!([#inner_ty; #size_expr])
        }
        spade_hir::TypeSpec::Inverted(inner) => {
            spade_type_to_syn_type(inner, module_nesting)
        }
        spade_hir::TypeSpec::Wire(inner) => {
            let inner_ty = spade_type_to_syn_type(inner, module_nesting);
            syn::parse_quote!(&#inner_ty)
        }
        spade_hir::TypeSpec::TraitSelf(_) => syn::parse_quote!(Self),
        spade_hir::TypeSpec::Wildcard(_) => syn::parse_quote!(_),
    }
}
