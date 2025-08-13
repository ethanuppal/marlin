// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{env, fs, iter, str};

use camino::Utf8PathBuf;
use marlin_verilator::PortDirection;
use marlin_verilog_macro_builder::{MacroArgs, build_verilated_struct};
use proc_macro::TokenStream;
use veryl_parser::{
    Parser,
    veryl_grammar_trait::{
        FactorTypeGroup, FirstToken, ModuleDeclaration,
        PortDeclarationGroupGroup, PortDeclarationItemGroup, ScalarTypeGroup,
    },
    veryl_walker::VerylWalker,
};

fn search_for_veryl_toml(mut start: Utf8PathBuf) -> Option<Utf8PathBuf> {
    while start.parent().is_some() {
        if start.join("Veryl.toml").is_file() {
            return Some(start.join("Veryl.toml"));
        }
        start.pop();
    }
    None
}

struct ModuleFinder<'args, 'source> {
    args: &'args MacroArgs,
    source_code: &'source str,
    look_for: String,
    found: Option<Vec<(String, usize, usize, PortDirection)>>,
    error: Option<syn::Error>,
}

impl VerylWalker for ModuleFinder<'_, '_> {
    fn module_declaration(&mut self, module: &ModuleDeclaration) {
        let name_token = &module.identifier.identifier_token.token;
        if &self.source_code.as_bytes()[name_token.pos as usize
            ..(name_token.pos + name_token.length) as usize]
            == self.look_for.as_bytes()
        {
            if let Some(port_declarations) = module
                .module_declaration_opt2
                .as_ref()
                .and_then(|opt2| {
                    opt2.port_declaration.port_declaration_opt.as_ref()
                })
                .map(|opt| &opt.port_declaration_list)
            {
                let veryl_ports =
                    iter::once(&port_declarations.port_declaration_group)
                        .chain(
                            port_declarations
                                .port_declaration_list_list
                                .iter()
                                .map(|after_first| {
                                    &after_first.port_declaration_group
                                }),
                        ).filter_map(|group|
                            match &*group.port_declaration_group_group {
                                PortDeclarationGroupGroup::LBracePortDeclarationListRBrace(_) => None,
                                PortDeclarationGroupGroup::PortDeclarationItem(port_declaration_group_group_port_declaration_item) => Some(port_declaration_group_group_port_declaration_item),
                            }
                        ).map(|item| {

                        let item = &item.port_declaration_item;
                        let port_name_token = item.identifier.identifier_token.token;
                        let port_name_bytes = &self.source_code.as_bytes()[port_name_token.pos as usize..(port_name_token.pos + port_name_token.length) as usize];
                        let port_name_str = str::from_utf8(port_name_bytes).expect("Veryl bug: Veryl identifier had invalid byte range (invalid UTF-8)");

                        (port_name_str, &item.port_declaration_item_group)
                    });

                let mut ports = vec![];
                for (port_name, port_type) in veryl_ports {
                    match &**port_type {
                        PortDeclarationItemGroup::PortTypeConcrete(
                            port_declaration_item_group_port_type_concrete,
                        ) => {
                            let concrete_type =
                                &port_declaration_item_group_port_type_concrete
                                    .port_type_concrete;

                            let port_direction = match &*concrete_type.direction {
                                veryl_parser::veryl_grammar_trait::Direction::Input(_) => PortDirection::Input,
                                veryl_parser::veryl_grammar_trait::Direction::Output(_) => PortDirection::Output,
                                veryl_parser::veryl_grammar_trait::Direction::Inout(_) => PortDirection::Inout,
                                veryl_parser::veryl_grammar_trait::Direction::Ref(_) => {
                                    self.error = Some(syn::Error::new_spanned(&self.args.name, format!("`{port_name}` is a ref port, which is currently not supported")));
                                    return;
                                },
                                veryl_parser::veryl_grammar_trait::Direction::Modport(_) => {
                                    self.error = Some(syn::Error::new_spanned(&self.args.name, format!("`{port_name}` is a modport, which is currently not supported")));
                                    return;
                                }
                                veryl_parser::veryl_grammar_trait::Direction::Import(_) => {
                                    self.error = Some(syn::Error::new_spanned(&self.args.name, format!("`{port_name}` is an import port, which is currently not supported")));
                                    return;
                                }
                            };

                            if concrete_type.array_type.array_type_opt.is_some()
                            {
                                self.error = Some(syn::Error::new_spanned(
                                    &self.args.name,
                                    format!(
                                        "`{port_name}` is an array, which is currently not supported"
                                    ),
                                ));
                                return;
                            }

                            let port_width = match &*concrete_type.array_type.scalar_type.scalar_type_group {
                                ScalarTypeGroup::UserDefinedTypeScalarTypeOpt(_scalar_type_group_user_defined_type_scalar_type_opt) => todo!("What is UserDefinedTypeScalarTypeOpt"),
                                ScalarTypeGroup::FactorType(scalar_type_group_factor_type) => {
                                    match &*scalar_type_group_factor_type.factor_type.factor_type_group {
                                        FactorTypeGroup::VariableTypeFactorTypeOpt(factor_type_group_variable_type_factor_type_opt) => {
                                            if let Some(factor_type) = factor_type_group_variable_type_factor_type_opt.factor_type_opt.as_ref() {
                                                factor_type.width.expression.token().to_string().parse::<usize>().expect("Veryl bug: parsed number but cannot convert to usize") - 1
                                            //match &*factor_type.width.expression fixed_type {
                                            //    FixedType::U32(fixed_type_u32) => &fixed_type_u32.u32.u32_token,
                                            //    FixedType::U64(fixed_type_u64) => &fixed_type_u64.u64.u64_token,
                                            //    FixedType::I32(fixed_type_i32) => &fixed_type_i32.i32.i32_token,
                                            //    FixedType::I64(fixed_type_i64) => &fixed_type_i64.i64.i64_token,
                                            //    FixedType::F32(_)|
                                            //    FixedType::F64(_) => todo!("Cannot use float as width"),
                                            //    FixedType::Strin(_) => todo!("Cannot use string as width"),
                                            //}.to_string().parse::<usize>().expect("Veryl bug: parsed number but cannot convert to usize")
                                            } else {
                                                1
                                            }
                                            //match &*factor_type_group_variable_type_factor_type_opt.variable_type {
                                            //    VariableType::Logic(variable_type_logic) => {
                                            //
                                            //    },
                                            //    _ => {
                                            //        self.error = Some(syn::Error::new_spanned(
                                            //            &self.args.name,
                                            //            format!(
                                            //                "`{port_name}` does not have logic type, and only logic is supported right now"
                                            //            ),
                                            //        ));
                                            //        return;
                                            //    }
                                            //}
                                        },
                                        FactorTypeGroup::FixedType(_factor_type_group_fixed_type) => {
                                            todo!("What is FactorTypeGroup::FixedType?")
                                            //match &*factor_type_group_fixed_type.fixed_type {
                                            //    FixedType::U32(fixed_type_u32) => &fixed_type_u32.u32.u32_token,
                                            //    FixedType::U64(fixed_type_u64) => &fixed_type_u64.u64.u64_token,
                                            //    FixedType::I32(fixed_type_i32) => &fixed_type_i32.i32.i32_token,
                                            //    FixedType::I64(fixed_type_i64) => &fixed_type_i64.i64.i64_token,
                                            //    FixedType::F32(_)|
                                            //    FixedType::F64(_) => todo!("Cannot use float as width"),
                                            //    FixedType::Strin(_) => todo!("Cannot use string as width"),
                                            //}.to_string().parse::<usize>().expect("Veryl bug: parsed number but cannot convert to usize")
                                        },
                                    }
                                }
                            };

                            ports.push((
                                port_name.to_string(),
                                port_width,
                                0,
                                port_direction,
                            ));
                        }
                        PortDeclarationItemGroup::PortTypeAbstract(_) => {
                            self.error = Some(syn::Error::new_spanned(
                                &self.args.name,
                                format!(
                                    "Port `{port_name}` has abstract type and therefore cannot be interfaced with"
                                ),
                            ));
                            return;
                        }
                    }
                }

                self.found = Some(ports);
            }
        }
    }
}

#[proc_macro_attribute]
pub fn veryl(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as MacroArgs);

    let manifest_directory = Utf8PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("Please use CARGO"),
    );
    let Some(veryl_toml_path) = search_for_veryl_toml(manifest_directory)
    else {
        return syn::Error::new_spanned(
            args.source_path,
            "Could not find Veryl.toml",
        )
        .into_compile_error()
        .into();
    };

    let veryl_source_path = {
        let mut veryl_source_path = veryl_toml_path.clone();
        veryl_source_path.pop();
        veryl_source_path.join(args.source_path.value())
    };
    let source_code = match fs::read_to_string(&veryl_source_path) {
        Ok(contents) => contents,
        Err(error) => {
            return syn::Error::new_spanned(
                &args.source_path,
                format!(
                    "Failed to read source code file at {veryl_source_path}: {error}"
                ),
            )
            .into_compile_error()
            .into();
        }
    };

    let parser = match Parser::parse(&source_code, &veryl_source_path) {
        Ok(parser) => parser,
        Err(error) => {
            return syn::Error::new_spanned(
                &args.source_path,
                format!(
                    "[veryl-parser] Failed to parser source code file at {veryl_source_path}: {error}"
                ),
            )
            .into_compile_error()
            .into();
        }
    };

    let mut module_finder = ModuleFinder {
        args: &args,
        source_code: &source_code,
        look_for: args.name.value(),
        found: None,
        error: None,
    };
    module_finder.veryl(&parser.veryl);

    let ports = if let Some(ports) = module_finder.found {
        ports
    } else {
        return module_finder.error.expect("Marlin bug for Veryl integration: ModuleFinder exited without ports or error").into_compile_error().into();
    };

    let verilog_source_path = syn::LitStr::new(
        veryl_source_path.with_extension("sv").as_str(),
        args.source_path.span(),
    );

    //let verilog_module_prefix = veryl_source_path
    //    .file_stem()
    //    .map(|stem| format!("{}_", stem))
    //    .unwrap_or_default();
    let veryl_toml_contents = match fs::read_to_string(&veryl_toml_path) {
        Ok(contents) => contents,
        Err(error) => {
            return syn::Error::new_spanned(&args.source_path, format!("Could not read contents of Veryl.toml at project root {veryl_toml_path}: {error}")).into_compile_error().into();
        }
    };

    let veryl_toml: toml::Value = match toml::from_str(&veryl_toml_contents) {
        Ok(toml) => toml,
        Err(error) => {
            return syn::Error::new_spanned(&args.source_path, format!("Could not parse contents of Veryl.toml at project root {veryl_toml_path} as a TOML file: {error}")).into_compile_error().into();
        }
    };

    let Some(project_name) = veryl_toml
        .get("project")
        .and_then(|project| project.get("name"))
        .and_then(|name| name.as_str())
    else {
        return syn::Error::new_spanned(&args.source_path, format!("Could not read the project.name field of Veryl.toml at project root {veryl_toml_path}")).into_compile_error().into();
    };

    let verilog_module_name = syn::LitStr::new(
        &format!("{}_{}", project_name, args.name.value()),
        args.name.span(),
    );

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
