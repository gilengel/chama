extern crate proc_macro;

use generate::{produce_skip_plugin, produce};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error, ResultExt};
use quote::{quote, ToTokens};
use structs::{EditorPluginArg, GenericParam, PluginAttribute, EditorPluginArgs};
use syn::parse::Parser;
use syn::AttributeArgs;
use syn::{parse_macro_input, DeriveInput, NestedMeta, Type};

extern crate proc_macro2;
extern crate quote;
extern crate syn;

use syn::{ItemFn, Meta};

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

/// Checks if an editor plugin has the skip attribute. Used
/// to skip the ui generation for its properties.
///
/// ```
/// #[editor_plugin(skip)]
/// struct MyPlugin {}
/// ```
fn plugin_has_skip_attribute(args: &AttributeArgs) -> bool {
    args.iter().any(|x| {
        if let NestedMeta::Meta(meta) = x {
            if let Meta::Path(path) = meta {
                if path.is_ident("skip") {
                    return true;
                }
            }
        }

        false
    })
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

fn derive(ast: &syn::DeriveInput) -> GenericParam {
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
                            .parse2(quote! { __enabled: bool })
                            .unwrap(),
                    );

                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! { __execution_behaviour: rust_internal::PluginExecutionBehaviour })
                            .unwrap(),
                    );
                }
                _ => (),
            }

            let derive_tokenstream = if !args.args.contains(&EditorPluginArg::Skip) {
                quote! {
                    #[derive(rust_macro::PluginWithOptions)]
                    #[type_trait(#generic_type, #execution_behaviour)]
                }
            } else {
                quote! {
                    #[derive(rust_macro::SkipPluginWithOptions)]
                    #[type_trait(#generic_type, #execution_behaviour)]
                }
            };

            return quote! {
                #derive_tokenstream
                #ast
            }
            .into();
        }
        _ => abort!(ast, "`add_field` has to be used with structs "),
    }
}

#[proc_macro_error]
#[proc_macro_derive(SkipPluginWithOptions, attributes(skip, option, type_trait))]
pub fn skip_plugin_with_options(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for plugin");
    let attrs: Vec<PluginAttribute> = parse_attrs(&ast);

    let param = derive(&ast);

    // Build the impl
    let gen = produce_skip_plugin(&ast, attrs, param.ty, param.execution_behaviour);

    // Return the generated impl
    gen.into()
}

#[proc_macro_error]
#[proc_macro_derive(PluginWithOptions, attributes(skip, option, type_trait))]
pub fn plugin_with_options(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for plugin");

    let param = derive(&ast);

    let attrs: Vec<PluginAttribute> = parse_attrs(&ast);

    // Build the impl
    let gen = produce(&ast, attrs, param.ty, param.execution_behaviour);

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

#[proc_macro_attribute]
#[proc_macro_error]
pub fn data_source(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_parsed = syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated
        .parse(args)
        .unwrap(); // Better to turn it into a `compile_error!()`

    let mut var_names: Vec<Ident> = vec![];
    let mut var_types: Vec<Ident> = vec![];
    let mut ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            match &mut struct_data.fields {
                syn::Fields::Named(fields) => {
                    for data_type_arg in args_parsed {
                        let data_type_arg = data_type_arg.segments[0].ident.clone();

                        let name = Ident::new(
                            &format!("{}s", data_type_arg.to_string().to_lowercase()).to_string(),
                            Span::call_site(),
                        );

                        var_names.push(name.clone());
                        var_types.push(data_type_arg.clone());

                        fields.named.push(
                            syn::Field::parse_named
                                .parse2(quote! { pub #name: HashMap<Uuid, #data_type_arg> })
                                .unwrap_or_abort(),
                        );
                    }
                }
                _ => (),
            }

            /*
            return quote! {
                #ast
            }.into();
            */
        }
        _ => panic!("`add_field` has to be used with structs "),
    }

    let mut get_all_fn: Vec<TokenStream2> = vec![];
    let mut get_single_fn: Vec<TokenStream2> = vec![];
    let mut get_multiple_by_id_fn: Vec<TokenStream2> = vec![];
    let mut get_multiple_at_position_fn: Vec<TokenStream2> = vec![];
    for (i, _) in var_names.iter().enumerate() {
        get_all_fn.push(implement_get_all(
            var_names[i].clone(),
            var_types[i].clone(),
        ));
        get_single_fn.push(implement_get_single(
            var_names[i].clone(),
            var_types[i].clone(),
        ));
        get_multiple_by_id_fn.push(implement_get_multiple_by_id(
            var_names[i].clone(),
            var_types[i].clone(),
        ));

        get_multiple_at_position_fn.push(implement_get_at_position(
            var_names[i].clone(),
            var_types[i].clone(),
        ));
    }

    let result = quote! {
        use std::collections::HashMap;
        use uuid::Uuid;
        use geo_types::Coordinate;

        //#[derive(Debug)]
        #ast

        impl #struct_name {

            pub fn new() -> Self {
                #struct_name { #( #var_names: HashMap::new()),* }
            }

            #( #get_all_fn )*

            #( #get_single_fn )*

            #( #get_multiple_by_id_fn )*

            #( #get_multiple_at_position_fn )*
        }
    };

    result.into()
}

fn function_name(var_name: &Ident, suffix: String, mutable: bool) -> Ident {
    Ident::new(
        &format!(
            "{}_{}{}",
            var_name.to_string(),
            suffix,
            if mutable { "_mut" } else { "" }
        )
        .to_string(),
        Span::call_site(),
    )
}

fn implement_get_all(var_name: Ident, var_type: Ident) -> TokenStream2 {
    let var_name_mut = Ident::new(
        &format!("{}_mut", var_name.to_string()).to_string(),
        Span::call_site(),
    );
    quote! {
        pub fn #var_name(&self) -> &HashMap<Uuid, #var_type> {
            &self.#var_name
        }

        pub fn #var_name_mut(&mut self) -> &mut HashMap<Uuid, #var_type> {
            &mut self.#var_name
        }
    }
}

fn implement_get_single(var_name: Ident, var_type: Ident) -> TokenStream2 {
    let single_var_name = var_name.to_string();
    let single_var_name: &str = &single_var_name[0..single_var_name.len() - 1];

    let single_var_name = Ident::new(&single_var_name, Span::call_site());
    let single_var_name_mut = Ident::new(
        &format!("{}_mut", single_var_name).to_string(),
        Span::call_site(),
    );
    quote! {
        pub fn #single_var_name(&self, id: &Uuid) -> Option<&#var_type> {
            if self.#var_name.contains_key(id) {
                return Some(self.#var_name.get(id).unwrap());
            }

            None
        }

        pub fn #single_var_name_mut(&mut self, id: &Uuid) -> Option<&mut #var_type> {
            if self.#var_name.contains_key(id) {
                return Some(self.#var_name.get_mut(id).unwrap());
            }

            None
        }
    }
}

fn implement_get_multiple_by_id(var_name: Ident, var_type: Ident) -> TokenStream2 {
    let fn_name = function_name(&var_name, "by_ids".to_string(), false);
    let fn_name_mut = function_name(&var_name, "by_ids".to_string(), true);

    quote! {
        pub fn #fn_name <'a>(
            &'a self,
            ids: &'a Vec<Uuid>,
        ) -> impl Iterator<Item = &'a #var_type> {
            self.#var_name
                .values()
                .filter(|element| ids.contains(&element.id()))
        }


        pub fn #fn_name_mut <'a>(
            &'a mut self,
            ids: &'a Vec<Uuid>,
        ) -> impl Iterator<Item = &'a mut #var_type> {
            self.#var_name
                .values_mut()
                .filter(|element| ids.contains(&element.id()))
        }

    }
}

fn implement_get_at_position(var_name: Ident, var_type: Ident) -> TokenStream2 {
    let fn_name = function_name(&var_name, "at_position".to_string(), false);
    let fn_name_mut = function_name(&var_name, "at_position".to_string(), true);

    quote! {
        pub fn #fn_name <'a>(
            &'a self,
            position: &Coordinate<f64>,
            offset: f64
        ) -> Option<&'a #var_type> {
            for (id, element) in &self.#var_name {
                if element.position().euclidean_distance(position) < offset {
                    return Some(*id);
                }
            }

            None
        }

        pub fn #fn_name_mut <'a>(
            &'a mut self,
            position: &Coordinate<f64>,
            offset: f64
        ) -> Option<&'a mut #var_type> {
            for (id, element) in &mut self.#var_name {
                if element.position().euclidean_distance(position) < offset {
                    return Some(*id);
                }
            }

            None
        }
    }
}
