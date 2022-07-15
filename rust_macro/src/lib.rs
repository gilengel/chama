extern crate proc_macro;

use generate::produce;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error, ResultExt};
use quote::{quote, ToTokens};
use structs::{EditorPluginArg, EditorPluginArgs, GenericParam, PluginAttribute};
use syn::parse::Parser;

use syn::{parse_macro_input, DeriveInput, ItemFn, Type};

extern crate proc_macro2;
extern crate quote;
extern crate syn;

use crate::parse::parse_attrs;

mod generate;
mod parse;
mod structs;

/// Entry point for your editor. Currently it is only a wrapper around wasm_bindgen
#[proc_macro_attribute]
pub fn launch(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    //let attrs = parse_macro_input!(attrs as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);

    quote! {
        #[wasm_bindgen(start)]
        pub #input
    }
    .into()
}

fn attribute_type(ty: Type) -> TokenStream2 {
    match ty {
        Type::Path(path) => path.into_token_stream(),
        _ => todo!(),
    }
}

fn plugin_generic_type(args: &Vec<EditorPluginArg>) -> Ident {
    for arg in args {
        if let EditorPluginArg::SpecificTo(x) = arg {
            return x.clone();
        }
    }

    Ident::new("Data", Span::call_site())
}

fn plugin_execution_behaviour(args: &Vec<EditorPluginArg>) -> Ident {
    for arg in args {
        if let EditorPluginArg::ExecutionBehaviour(x) = arg {
            return x.clone();
        }
    }

    Ident::new("Always", Span::call_site())
}

fn derive_plugin_params(ast: &syn::DeriveInput) -> GenericParam {
    let attribute = ast
        .attrs
        .iter()
        .filter(|a| a.path.segments.len() == 1 && a.path.segments[0].ident == "type_trait")
        .nth(0)
        .expect("type_trait attribute required for deriving TypeTrait!");

    let parameter: GenericParam =
        syn::parse2(attribute.tokens.clone()).expect("Invalid type_trait attribute!");

    parameter
}

fn derive_plugin_skip(ast: &syn::DeriveInput) -> bool {
    ast.attrs
        .iter()
        .filter(|a| a.path.segments.len() == 1 && a.path.segments[0].ident == "skip")
        .nth(0)
        != None
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn editor_plugin(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let args = parse_macro_input!(args as EditorPluginArgs);
    let generic_type = plugin_generic_type(&args.args);
    let execution_behaviour = plugin_execution_behaviour(&args.args);

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            match &mut struct_data.fields {
                syn::Fields::Named(fields) => {
                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! { __enabled: Rc<RefCell<bool>> })
                            .unwrap_or_else(|_| abort!(fields, "Not possible to add to fields")),
                    );

                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! { __execution_behaviour: rust_internal::PluginExecutionBehaviour })
                            .unwrap_or_else(|_| abort!(fields, "Not possible to add to fields")),
                    );
                }
                _ => (),
            }

            let skip = match args.args.contains(&EditorPluginArg::Skip) {
                true => quote! { #[skip] },
                false => quote! {},
            };

            return quote! {
                #[derive(rust_macro::PluginWithOptions)]
                #[type_trait(#generic_type, #execution_behaviour)]
                #skip

                #ast
            }
            .into();
        }
        _ => abort!(ast, "`add_field` has to be used with structs "),
    }
}

#[proc_macro_error]
#[proc_macro_derive(PluginWithOptions, attributes(skip, option, type_trait))]
pub fn plugin_with_options(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for plugin");

    let param = derive_plugin_params(&ast);
    let attrs: Vec<PluginAttribute> = parse_attrs(&ast);

    let skip_ui_gen = derive_plugin_skip(&ast);

    // Build the impl
    let gen = produce(&ast, attrs, &param, skip_ui_gen);

    // Return the generated impl
    gen.into()
}

// DATASOURCE

#[proc_macro_derive(ElementId)]
pub fn derive_id_trait_functions(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);

    let name = input.ident;

    let modified = quote! {
        impl SetId for #name {
            fn set_id(&mut self, id: Uuid) {
                self.id = id;
            }
        }

        impl Id for #name {
            fn id(&self) -> Uuid {
                self.id
            }
        }
    };
    TokenStream::from(modified)
}