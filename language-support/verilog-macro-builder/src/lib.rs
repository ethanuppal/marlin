// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::HashMap, path::Path};

use marlin_verilator::PortDirection;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use sv_parser::{self as sv, Locate, RefNode, unwrap_node};

mod util;

pub struct MacroArgs {
    pub source_path: syn::LitStr,
    pub name: syn::LitStr,

    pub clock_port: Option<syn::LitStr>,
    pub reset_port: Option<syn::LitStr>,
}

impl syn::parse::Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        syn::custom_keyword!(src);
        syn::custom_keyword!(name);

        syn::custom_keyword!(clock);
        syn::custom_keyword!(reset);
        input.parse::<src>()?;
        input.parse::<syn::Token![=]>()?;
        let source_path = input.parse::<syn::LitStr>()?;

        input.parse::<syn::Token![,]>()?;

        input.parse::<name>()?;
        input.parse::<syn::Token![=]>()?;
        let name = input.parse::<syn::LitStr>()?;

        let mut clock_port = None;
        let mut reset_port = None;
        while input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;

            let lookahead = input.lookahead1();
            if lookahead.peek(clock) {
                input.parse::<clock>()?;
                input.parse::<syn::Token![=]>()?;
                clock_port = Some(input.parse::<syn::LitStr>()?);
            } else if lookahead.peek(reset) {
                input.parse::<reset>()?;
                input.parse::<syn::Token![=]>()?;
                reset_port = Some(input.parse::<syn::LitStr>()?);
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(Self {
            source_path,
            name,
            clock_port,
            reset_port,
        })
    }
}

pub fn build_verilated_struct(
    macro_name: &str,
    top_name: syn::LitStr,
    source_path: syn::LitStr,
    verilog_ports: Vec<(String, usize, usize, PortDirection)>,
    clock_port: Option<syn::LitStr>,
    reset_port: Option<syn::LitStr>,
    item: TokenStream,
) -> TokenStream {
    let crate_name = format_ident!("{}", macro_name);
    let item = match syn::parse::<syn::ItemStruct>(item.into()) {
        Ok(item) => item,
        Err(error) => {
            return error.into_compile_error();
        }
    };

    let mut struct_members = vec![];

    let mut preeval_impl = vec![];
    let mut posteval_impl = vec![];

    let mut other_impl = vec![];

    let mut verilated_model_ports_impl = vec![];
    let mut verilated_model_init_impl = vec![];
    let mut verilated_model_init_self = vec![];

    let mut dynamic_read_arms = vec![];
    let mut dynamic_pin_arms = vec![];

    verilated_model_init_impl.push(quote! {
        let new_model: extern "C" fn() -> *mut std::ffi::c_void =
            *unsafe { library.get(concat!("ffi_new_V", #top_name).as_bytes()) }
                .expect("failed to get symbol");
        let model = (new_model)();

        let eval_model: extern "C" fn(*mut std::ffi::c_void) =
            *unsafe { library.get(concat!("ffi_V", #top_name, "_eval").as_bytes()) }
                .expect("failed to get symbol");
    });
    verilated_model_init_self.push(quote! {
        eval_model,
        model,
        _marker: std::marker::PhantomData
    });

    for (port_name, port_msb, port_lsb, port_direction) in verilog_ports {
        if port_name.chars().any(|c| c == '\\' || c == ' ') {
            return syn::Error::new_spanned(
                top_name,
                "Escaped module names are not supported",
            )
            .into_compile_error();
        }

        let port_width = port_msb + 1 - port_lsb;

        let port_type_name = if port_width <= 8 {
            quote! { CData }
        } else if port_width <= 16 {
            quote! { SData }
        } else if port_width <= 32 {
            quote! { IData }
        } else if port_width <= 64 {
            quote! { QData }
        } else {
            return syn::Error::new_spanned(
                source_path,
                format!("Port `{port_name}` is wider than supported right now"),
            )
            .into_compile_error();
        };
        let port_type = quote! { #crate_name::__reexports::verilator::types::#port_type_name };

        let port_name_ident = format_ident!("{}", port_name);
        let port_documentation = syn::LitStr::new(
            &format!(
                "Corresponds to Verilog `{port_direction} {port_name}[{port_msb}:{port_lsb}]`."
            ),
            top_name.span(),
        );
        struct_members.push(quote! {
            #[doc = #port_documentation]
            pub #port_name_ident: #port_type
        });
        verilated_model_init_self.push(quote! {
            #port_name_ident: 0 as _
        });

        let port_name_literal = syn::LitStr::new(&port_name, top_name.span());

        match port_direction {
            PortDirection::Input => {
                let setter = format_ident!("pin_{}", port_name);
                struct_members.push(quote! {
                    #[doc(hidden)]
                    #setter: extern "C" fn(*mut std::ffi::c_void, #port_type)
                });
                preeval_impl.push(quote! {
                    (self.#setter)(self.model, self.#port_name_ident);
                });

                if let Some(clock_port) = &clock_port {
                    if clock_port.value().as_str() == port_name {
                        other_impl.push(quote! {
                            pub fn tick(&mut self) {
                                self.#port_name = 1 as _;
                                self.eval();
                                self.#port_name = 0 as _;
                                self.eval();
                            }
                        });
                    }
                }

                if let Some(_reset_port) = &reset_port {
                    todo!("reset ports");
                }

                verilated_model_init_impl.push(quote! {
                    let #setter: extern "C" fn(*mut std::ffi::c_void, #port_type) =
                        *unsafe { library.get(concat!("ffi_V", #top_name, "_pin_", #port_name).as_bytes()) }
                            .expect("failed to get symbol");
                });
                verilated_model_init_self.push(quote! { #setter });

                dynamic_pin_arms.push(quote! {
                    #port_name_literal => {
                        if let #crate_name::__reexports::verilator::dynamic::VerilatorValue::#port_type_name(inner) = value {
                            self.#port_name_ident = inner;
                        } else {
                            return Err(
                                #crate_name::__reexports::verilator::dynamic::DynamicVerilatedModelError::InvalidPortWidth {
                                    top_module: Self::name().to_string(),
                                    port: port,
                                    width: #port_width as _,
                                    attempted_lower: 0,
                                    attempted_higher: value.width()
                                },
                            );
                        }
                    }
                });
            }
            PortDirection::Output => {
                let getter = format_ident!("read_{}", port_name);
                struct_members.push(quote! {
                    #[doc(hidden)]
                    #getter: extern "C" fn(*mut std::ffi::c_void) -> #port_type
                });
                posteval_impl.push(quote! {
                    self.#port_name_ident = (self.#getter)(self.model);
                });

                verilated_model_init_impl.push(quote! {
                    let #getter: extern "C" fn(*mut std::ffi::c_void) -> #port_type =
                        *unsafe { library.get(concat!("ffi_V", #top_name, "_read_", #port_name).as_bytes()) }
                            .expect("failed to get symbol");
                });
                verilated_model_init_self.push(quote! { #getter });

                dynamic_read_arms.push(quote! {
                    #port_name_literal => Ok(self.#port_name_ident.into())
                });
            }
            _ => todo!("Unhandled port direction"),
        }

        let verilated_model_port_direction = match port_direction {
            PortDirection::Input => {
                quote! { #crate_name::__reexports::verilator::PortDirection::Input }
            }
            PortDirection::Output => {
                quote! { #crate_name::__reexports::verilator::PortDirection::Output }
            }
            _ => todo!("Other port directions"),
        };

        verilated_model_ports_impl.push(quote! {
            (#port_name, #port_msb, #port_lsb, #verilated_model_port_direction)
        });
    }

    struct_members.push(quote! {
        #[doc(hidden)]
        eval_model: extern "C" fn(*mut std::ffi::c_void)
    });

    let struct_name = item.ident;
    let vis = item.vis;
    let port_count = verilated_model_ports_impl.len();
    quote! {
        #vis struct #struct_name<'ctx> {
            #[doc(hidden)]
            vcd_api: Option<#crate_name::__reexports::verilator::vcd::__private::VcdApi>,
            #[doc(hidden)]
            opened_vcd: bool,
            #(#struct_members),*,
            #[doc = "# Safety\nThe Rust binding to the model will not outlive the dynamic library context (with lifetime `'ctx`) and is dropped when this struct is."]
            #[doc(hidden)]
            model: *mut std::ffi::c_void,
            #[doc(hidden)]
            _marker: std::marker::PhantomData<&'ctx ()>,
            #[doc(hidden)]
            _unsend_unsync: std::marker::PhantomData<(std::cell::Cell<()>, std::sync::MutexGuard<'static, ()>)>
        }

        impl<'ctx> #struct_name<'ctx> {
            #[doc = "Equivalent to the Verilator `eval` method."]
            pub fn eval(&mut self) {
                #(#preeval_impl)*
                (self.eval_model)(self.model);
                #(#posteval_impl)*
            }

            pub fn open_vcd(
                &mut self,
                path: impl std::convert::AsRef<std::path::Path>,
            ) -> #crate_name::__reexports::verilator::vcd::Vcd<'ctx> {
                let path = path.as_ref();
                if let Some(vcd_api) = &self.vcd_api {
                    if self.opened_vcd {
                        panic!("Verilator does not support opening multiple VCD traces (see issue #5813). You can instead split the already-opened VCD.");
                    }
                    let c_path = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("Failed to convert provided VCD path to C string");
                    let vcd_ptr = (vcd_api.open_trace)(self.model, c_path.as_ptr());
                    self.opened_vcd = true;
                    #crate_name::__reexports::verilator::vcd::__private::new_vcd(
                        vcd_ptr,
                        vcd_api.dump,
                        vcd_api.open_next,
                        vcd_api.flush,
                        vcd_api.close_and_delete
                    )
                } else {
                    #crate_name::__reexports::verilator::vcd::__private::new_vcd_useless()
                }
            }

            #(#other_impl)*
        }

        impl<'ctx> #crate_name::__reexports::verilator::AsVerilatedModel<'ctx> for #struct_name<'ctx> {
            fn name() -> &'static str {
                #top_name
            }

            fn source_path() -> &'static str {
                #source_path
            }

            fn ports() -> &'static [(&'static str, usize, usize, #crate_name::__reexports::verilator::PortDirection)] {
                static PORTS: [(&'static str, usize, usize, #crate_name::__reexports::verilator::PortDirection); #port_count] = [#(#verilated_model_ports_impl),*];
                &PORTS
            }

            fn init_from(library: &'ctx #crate_name::__reexports::libloading::Library, tracing_enabled: bool) -> Self {
                #(#verilated_model_init_impl)*

                let vcd_api =
                    if tracing_enabled {
                        use #crate_name::__reexports::verilator::vcd::__private::VcdApi;

                        let open_trace: extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_char) -> *mut std::ffi::c_void =
                            *unsafe { library.get(concat!("ffi_V", #top_name, "_open_trace").as_bytes()).expect("failed to get open_trace symbol") };
                        let dump: extern "C" fn(*mut std::ffi::c_void, u64) =
                            *unsafe { library.get(b"ffi_VerilatedVcdC_dump").expect("failed to get dump symbol") };
                        let open_next: extern "C" fn(*mut std::ffi::c_void, bool) =
                            *unsafe { library.get(b"ffi_VerilatedVcdC_open_next").expect("failed to get open_next symbol") };
                        let flush: extern "C" fn(*mut std::ffi::c_void) =
                            *unsafe { library.get(b"ffi_VerilatedVcdC_flush").expect("failed to get flush symbol") };
                        let close_and_delete: extern "C" fn(*mut std::ffi::c_void) =
                            *unsafe { library.get(b"ffi_VerilatedVcdC_close_and_delete").expect("failed to get close_and_delete symbol") };
                        Some(VcdApi { open_trace, dump, open_next, flush, close_and_delete })
                    } else {
                        None
                    };

                Self {
                    vcd_api,
                    opened_vcd: false,
                    #(#verilated_model_init_self),*,
                    _unsend_unsync: std::marker::PhantomData
                }
            }

            unsafe fn model(&self) -> *mut std::ffi::c_void {
                self.model
            }
        }

        impl<'ctx> #crate_name::__reexports::verilator::AsDynamicVerilatedModel<'ctx> for #struct_name<'ctx> {
            fn read(
                &self,
                port: impl Into<String>,
            ) -> Result<#crate_name::__reexports::verilator::dynamic::VerilatorValue, #crate_name::__reexports::verilator::dynamic::DynamicVerilatedModelError> {
                use #crate_name::__reexports::verilator::AsVerilatedModel;

                let port = port.into();

                match port.as_str() {
                    #(#dynamic_read_arms,)*
                    _ => Err(#crate_name::__reexports::verilator::dynamic::DynamicVerilatedModelError::NoSuchPort {
                        top_module: Self::name().to_string(),
                        port,
                        source: None,
                    })
                }
            }

            fn pin(
                &mut self,
                port: impl Into<String>,
                value: impl Into<#crate_name::__reexports::verilator::dynamic::VerilatorValue>,
            ) -> Result<(), #crate_name::__reexports::verilator::dynamic::DynamicVerilatedModelError> {
                use #crate_name::__reexports::verilator::AsVerilatedModel;

                let port = port.into();
                let value = value.into();

                match port.as_str() {
                    #(#dynamic_pin_arms,)*
                    _ => {
                        return Err(#crate_name::__reexports::verilator::dynamic::DynamicVerilatedModelError::NoSuchPort {
                            top_module: Self::name().to_string(),
                            port,
                            source: None,
                        });
                    }
                }

                #[allow(unreachable_code)]
                Ok(())
            }
        }
    }
}

pub fn parse_verilog_ports(
    top_name: &syn::LitStr,
    source_path: &syn::LitStr,
    verilog_source_path: &Path,
) -> Result<Vec<(String, usize, usize, PortDirection)>, proc_macro2::TokenStream>
{
    let defines = HashMap::new();
    let (ast, _) =
        match sv::parse_sv(verilog_source_path, &defines, &["."], false, false)
        {
            Ok(result) => result,
            Err(error) => {
                return Err(syn::Error::new_spanned(
                source_path,
                error.to_string()
                    + " (Try checking, for instance, that the file exists.)",
            )
            .into_compile_error());
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
            if id == top_name.value().as_str() {
                Some(module)
            } else {
                None
            }
        }
        _ => None,
    }) else {
        return Err(syn::Error::new_spanned(
            top_name,
            format!(
                "Could not find module declaration for `{}` in {}",
                top_name.value(),
                source_path.value()
            ),
        )
        .into_compile_error());
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

                let (port_direction_node, port_type) = net
                    .nodes
                    .0
                    .as_ref()
                    .and_then(|maybe_net_header| match maybe_net_header {
                        sv::NetPortHeaderOrInterfacePortHeader::NetPortHeader(net_port_header) => {
                            net_port_header.nodes.0.as_ref().map(|d| (d, &net_port_header.nodes.1))
                        }
                        _ => todo!("Other port header"),
                    })
                    .ok_or_else(|| {
                        syn::Error::new_spanned(
                            source_path,
                            format!(
                                "Port `{port_name}` has no supported direction (`input` or `output`)"
                            ),
                        )
                        .into_compile_error()
                    })?;

                let dimensions: &[sv::PackedDimension] = match port_type {
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
                    sv::NetPortType::NetTypeIdentifier(_)
                    | sv::NetPortType::Interconnect(_) => {
                        todo!("Port type not yet implemented for net ports")
                    }
                };

                let port_info = match process_port_common(
                    &ast,
                    top_name,
                    port_name,
                    dimensions,
                    port_direction_node,
                ) {
                    Ok(port_info) => port_info,
                    Err(error) => {
                        return Err(error.into_compile_error());
                    }
                };
                ports.push(port_info);
            }

            sv::AnsiPortDeclaration::Variable(var) => {
                let port_name = ast.get_str_trim(&var.nodes.1.nodes.0).expect(
                    "Port identifier could not be traced back to source code",
                );

                let (port_direction_node, port_type) = var
                    .nodes
                    .0
                    .as_ref()
                    .and_then(|header| {
                        header.nodes.0.as_ref().map(|d| (d, &header.nodes.1))
                    })
                    .ok_or_else(|| {
                        syn::Error::new_spanned(
                            source_path,
                            format!(
                                "Port `{port_name}` has no supported direction (`input` or `output`)"
                            ),
                        )
                        .into_compile_error()
                    })?;

                let dimensions: &[sv::PackedDimension] = match &port_type
                    .nodes
                    .0
                {
                    sv::VarDataType::DataType(data_type) => {
                        match &**data_type {
                            sv::DataType::Vector(data_type_vector) => {
                                &data_type_vector.nodes.2
                            }
                            other => todo!("Unsupported data type {:?}", other),
                        }
                    }
                    sv::VarDataType::Var(var_data_type_var) => {
                        match &var_data_type_var.nodes.1 {
                            sv::DataTypeOrImplicit::DataType(data_type) => {
                                match &**data_type {
                                    sv::DataType::Vector(data_type_vector) => {
                                        &data_type_vector.nodes.2
                                    }
                                    other => todo!(
                                        "Unsupported data type (in the VarDataType>DataTypeOrImplicit>DataType branch) {:?}",
                                        other
                                    ),
                                }
                            }
                            sv::DataTypeOrImplicit::ImplicitDataType(
                                implicit_data_type,
                            ) => &implicit_data_type.nodes.1,
                        }
                    }
                };

                let port_info = match process_port_common(
                    &ast,
                    top_name,
                    port_name,
                    dimensions,
                    port_direction_node,
                ) {
                    Ok(port_info) => port_info,
                    Err(error) => {
                        return Err(error.into_compile_error());
                    }
                };
                ports.push(port_info);
            }
            _ => todo!("Other types of ports"),
        }
    }

    Ok(ports)
}

fn process_port_common(
    ast: &sv::SyntaxTree,
    top_name: &syn::LitStr,
    port_name: &str,
    dimensions: &[sv::PackedDimension],
    port_direction_node: &sv::PortDirection,
) -> Result<(String, usize, usize, PortDirection), syn::Error> {
    if port_name.chars().any(|c| c == '\\' || c == ' ') {
        return Err(syn::Error::new_spanned(
            top_name,
            "Escaped module names are not supported",
        ));
    }

    let (port_msb, port_lsb) = match dimensions.len() {
        0 => (0, 0),
        1 => match &dimensions[0] {
            sv::PackedDimension::Range(packed_dimension_range) => {
                let range = &packed_dimension_range.nodes.0.nodes.1.nodes;
                (
                    util::evaluate_numeric_constant_expression(ast, &range.0),
                    util::evaluate_numeric_constant_expression(ast, &range.2),
                )
            }
            _ => todo!("Unsupported dimension type"),
        },
        _ => todo!("Don't support multidimensional ports yet"),
    };

    let port_direction = match port_direction_node {
        sv::PortDirection::Input(_) => PortDirection::Input,
        sv::PortDirection::Output(_) => PortDirection::Output,
        sv::PortDirection::Inout(_) => PortDirection::Inout,
        sv::PortDirection::Ref(_) => {
            todo!("Reference port direction is not supported")
        }
    };

    Ok((port_name.to_string(), port_msb, port_lsb, port_direction))
}
