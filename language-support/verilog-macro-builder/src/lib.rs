// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use verilator::PortDirection;

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

    verilated_model_init_impl.push(quote! {
        let new_model: extern "C" fn() -> *mut #crate_name::__reexports::libc::c_void =
            *unsafe { library.get(concat!("ffi_new_V", #top_name).as_bytes()) }
                .expect("failed to get symbol");
        let model = (new_model)();


        let delete_model: extern "C" fn(*mut #crate_name::__reexports::libc::c_void) =
            *unsafe { library.get(concat!("ffi_delete_V", #top_name).as_bytes()) }
                .expect("failed to get symbol");

        let eval_model: extern "C" fn(*mut #crate_name::__reexports::libc::c_void) =
            *unsafe { library.get(concat!("ffi_V", #top_name, "_eval").as_bytes()) }
                .expect("failed to get symbol");
    });
    verilated_model_init_self.push(quote! {
        drop_model: delete_model,
        eval_model,
        model,
        _phantom: std::marker::PhantomData
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

        let port_type = if port_width <= 8 {
            quote! { #crate_name::__reexports::verilator::types::CData }
        } else if port_width <= 16 {
            quote! { #crate_name::__reexports::verilator::types::SData }
        } else if port_width <= 32 {
            quote! { #crate_name::__reexports::verilator::types::IData }
        } else if port_width <= 64 {
            quote! { #crate_name::__reexports::verilator::types::QData }
        } else {
            return syn::Error::new_spanned(
                source_path,
                format!(
                    "Port `{}` is wider than supported right now",
                    port_name
                ),
            )
            .into_compile_error();
        };

        let port_name_ident = format_ident!("{}", port_name);
        struct_members.push(quote! {
            pub #port_name_ident: #port_type
        });
        verilated_model_init_self.push(quote! {
            #port_name_ident: 0 as _
        });

        match port_direction {
            PortDirection::Input => {
                let setter = format_ident!("pin_{}", port_name);
                struct_members.push(quote! {
                            #setter: extern "C" fn(*mut #crate_name::__reexports::libc::c_void, #port_type)
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
                            let #setter: extern "C" fn(*mut #crate_name::__reexports::libc::c_void, #port_type) =
                                *unsafe { library.get(concat!("ffi_V", #top_name, "_pin_", #port_name).as_bytes()) }
                                    .expect("failed to get symbol");
                        });
                verilated_model_init_self.push(quote! { #setter });
            }
            PortDirection::Output => {
                let getter = format_ident!("read_{}", port_name);
                struct_members.push(quote! {
                            #getter: extern "C" fn(*mut #crate_name::__reexports::libc::c_void) -> #port_type
                        });
                posteval_impl.push(quote! {
                    self.#port_name_ident = (self.#getter)(self.model);
                });

                verilated_model_init_impl.push(quote! {
                            let #getter: extern "C" fn(*mut #crate_name::__reexports::libc::c_void) -> #port_type =
                                *unsafe { library.get(concat!("ffi_V", #top_name, "_read_", #port_name).as_bytes()) }
                                    .expect("failed to get symbol");
                        });
                verilated_model_init_self.push(quote! { #getter });
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
        drop_model: extern "C" fn(*mut #crate_name::__reexports::libc::c_void),
        eval_model: extern "C" fn(*mut #crate_name::__reexports::libc::c_void)
    });

    let struct_name = item.ident;
    let vis = item.vis;
    let port_count = verilated_model_ports_impl.len();
    quote! {
        #vis struct #struct_name<'ctx> {
            #(#struct_members),*,
            #[doc = "# Safety\nThe Rust binding to the model will not outlive the dynamic library context (with lifetime `'ctx`) and is dropped when this struct is."]
            model: *mut #crate_name::__reexports::libc::c_void,
            _phantom: std::marker::PhantomData<&'ctx ()>
        }

        impl #struct_name<'_> {
            pub fn eval(&mut self) {
                #(#preeval_impl)*
                (self.eval_model)(self.model);
                #(#posteval_impl)*
            }

            #(#other_impl)*
        }

        impl<'ctx> std::ops::Drop for #struct_name<'ctx> {
            fn drop(&mut self) {
                (self.drop_model)(self.model);
                self.model = std::ptr::null_mut();
            }
        }

        impl<'ctx> #crate_name::__reexports::verilator::VerilatedModel for #struct_name<'ctx> {
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

            fn init_from(library: &#crate_name::__reexports::libloading::Library) -> Self {
                #(#verilated_model_init_impl)*
                Self {
                    #(#verilated_model_init_self),*
                }
            }
        }
    }
}
