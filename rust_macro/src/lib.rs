extern crate proc_macro;

use generate::generate_option_element;
use proc_macro::TokenStream;
use proc_macro2::Delimiter;
use proc_macro2::{Group, Punct, Spacing, TokenStream as TokenStream2, TokenTree};
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error, ResultExt};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::Parser;
use syn::AttributeArgs;
use syn::{parse_macro_input, DeriveInput, Lit, NestedMeta, Type};

extern crate proc_macro2;
extern crate quote;
extern crate syn;

use syn::{DataStruct, Meta, ItemFn};

use crate::generate::{checkbox, generate_default_arm};
use crate::parse::parse_attrs;

mod generate;
mod parse;

/// Entry point for your editor. Currently it is only a wrapper around wasm_bindgen
#[proc_macro_attribute]
pub fn launch(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attrs as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);

    quote!{
        #[wasm_bindgen(start)]
        pub #input
    }.into()
}

fn attribute_type(ty: Type) -> TokenStream2 {
    match ty {
        Type::Path(path) => path.into_token_stream(),
        _ => todo!(),
    }
}

pub(crate) struct VisibleAttribute {
    pub name: Ident,
    pub label: Lit,
    pub description: Lit,
}

pub(crate) struct HiddenAttribute {
    pub name: Ident,
}

pub(crate) enum Attribute {
    Visible(VisibleAttribute),
    Hidden(HiddenAttribute),
}

type PluginAttribute = (Attribute, TokenStream2, Vec<Meta>);

/// Checks if an editor plugin has the skip attribute. Used
/// to skip the ui generation for its properties.
///
/// ```
/// #[editor_plugin(skip)]
/// struct MyPlugin {}
/// ```
fn plugin_has_skip_attribute(args: AttributeArgs) -> bool {
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

#[proc_macro_error]
#[proc_macro_attribute]
pub fn editor_plugin(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);

    let args = parse_macro_input!(args as AttributeArgs);
    let skip = plugin_has_skip_attribute(args);

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            match &mut struct_data.fields {
                syn::Fields::Named(fields) => {
                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! { __enabled: bool })
                            .unwrap(),
                    );
                }
                _ => (),
            }

            let derive_tokenstream = if !skip {
                quote! { #[derive(rust_macro::PluginWithOptions)] }
            } else {
                quote! { #[derive(rust_macro::SkipPluginWithOptions)] }
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
#[proc_macro_derive(SkipPluginWithOptions, attributes(skip, option))]
pub fn skip_plugin_with_options(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for plugin");

    let attrs: Vec<PluginAttribute> = parse_attrs(&ast);

    // Build the impl
    let gen = produce_skip_plugin(&ast, attrs);

    // Return the generated impl
    gen.into()
}

#[proc_macro_error]
#[proc_macro_derive(PluginWithOptions, attributes(skip, option))]
pub fn plugin_with_options(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for plugin");

    let attrs: Vec<PluginAttribute> = parse_attrs(&ast);

    // Build the impl
    let gen = produce(&ast, attrs);

    // Return the generated impl
    gen.into()
}

pub(crate) fn get_mandatory_meta_value<'a>(
    meta_attrs: &'a Vec<Meta>,
    identifier: &str,
) -> Option<&'a Lit> {
    if let Some(default_meta) = meta_attrs
        .iter()
        .find(|meta| meta.path().is_ident(identifier))
    {
        if let Meta::NameValue(e) = &default_meta {
            return Some(&e.lit);
        }
    }

    None
}

fn produce_default_impl(ast: &DeriveInput, attrs: &Vec<PluginAttribute>) -> TokenStream2 {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Used for the default impl function
    let mut defaults: Vec<TokenStream2> = vec![];
    defaults.push(quote! { __enabled: true });

    // Now for all attributes defined be the plugin developer
    for (attr, ty, metas) in attrs {
        match attr {
            Attribute::Visible(attr) => {
                defaults.push(generate_default_arm(&attr.name, &ty, &metas));
            }
            Attribute::Hidden(attr) => {
                defaults.push(generate_default_arm(&attr.name, &ty, &metas));
            }
        }
    }

    quote! {
        impl #impl_generics Default for #name #ty_generics #where_clause{
            fn default() -> Self {
                Self {
                    #(#defaults),*
                }
            }
        }
    }
}

fn produce_identifier_impl(name: &Ident) -> TokenStream2 {
    let name_str = name.to_string();
    quote! {
        fn identifier() -> &'static str where Self: Sized {
            #name_str
        }
    }
}

fn produce_enabled() -> TokenStream2 {
    quote! {
        fn enabled(&self) -> bool {
            self.__enabled
        }
    }
}

fn produce_as_any_impl() -> TokenStream2 {
    quote! {
        fn as_any(&self) -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    }
}

fn produce_ui_impl(ast: &DeriveInput, attrs: &Vec<PluginAttribute>) -> TokenStream2 {
    let name = &ast.ident;
    let name_str = name.to_string();

    let mut t = TokenStream2::new();
    t.extend(vec![
        TokenTree::Ident(Ident::new("html", Span::call_site())),
        TokenTree::Punct(Punct::new('!', Spacing::Alone)),
    ]);

    let mut callbacks: Vec<TokenStream2> = vec![];

    let plugin = name_str.clone();

    let mut inner = TokenStream2::new();

    // match arms used when a plugin received a message one of its properties was changed.
    // The match arm maps the message content to the corresponding property
    let mut arms: Vec<TokenStream2> = vec![];

    // List of the html elements
    let mut elements: Vec<TokenStream2> =
        vec![quote! { <div> }, quote! { <div><h2>{#name_str}</h2> }];

    // Enable checkbox for each plugin
    let enabled_checkbox = checkbox(&plugin.clone(), "__enabled", true);
    elements.push(enabled_checkbox.0);
    elements.push(quote! { </div> });
    callbacks.push(enabled_checkbox.1);

    arms.push(quote! { "__enabled" => { if let Some(value) = value.as_ref().downcast_ref::<bool>() { self.__enabled = *value } }});

    // Now for all attributes defined be the plugin developer
    for (attr, ty, metas) in attrs {
        if let Attribute::Visible(attr) = attr {
            let attr = generate_option_element(&plugin, attr, ty, metas);
            elements.push(attr.element);
            callbacks.push(attr.callback);
            arms.push(attr.arm);
        }
    }

    elements.push(quote! {</div>});
    inner.append_all(elements);
    t.extend(vec![TokenTree::Group(Group::new(Delimiter::Brace, inner))]);

    let gen = quote! {
        fn view_options(&self, ctx: &Context<App<Data, Modes>>) -> Html {
            #(#callbacks)*
            #t
        }

        fn update_property(&mut self, property: &str, value: Box<dyn Any>) {
            match property {
                #(#arms),*
                _ => { web_sys::console::log_1(&":(".into()); }
            }
        }
    };

    gen
}

fn produce_skip_plugin(ast: &DeriveInput, attrs: Vec<PluginAttribute>) -> TokenStream2 {
    // Is it a struct?
    if let syn::Data::Struct(DataStruct { .. }) = ast.data {
        let generics = &ast.generics;
        let (_, ty_generics, _) = generics.split_for_impl();

        let name = &ast.ident;

        // generated methods
        let identifier_impl = produce_identifier_impl(&ast.ident);
        let default_impl = produce_default_impl(ast, &attrs);
        let ui_impl = produce_ui_impl(ast, &attrs);
        let enabled_impl = produce_enabled();
        let produce_as_any_impl = produce_as_any_impl();

        let muu = quote! {
            use std::ops::Deref;
            use yew::{html, Html, Callback, Context};
            use rust_internal::ui::textbox::TextBox;
            use rust_internal::ui::{numberbox::NumberBox, checkbox::Checkbox};
            use crate::ui::app::App;
            use crate::ui::app::EditorMessages;
            use std::any::Any;

            impl<Data, Modes> crate::PluginWithOptions<Data, Modes> for #name #ty_generics where
                Data: Renderer + Default + 'static,
                Modes: Clone + std::cmp::PartialEq + Eq + std::hash::Hash + 'static, {

                #identifier_impl
                #enabled_impl
            }

            impl<Data> crate::plugins::plugin::AnyPlugin<Data> for #name #ty_generics where
                Data: Renderer + Default + 'static, {

                #produce_as_any_impl

            }

            #default_impl
        };

        muu
    } else {
        quote! {}
    }
}

fn produce(ast: &DeriveInput, attrs: Vec<PluginAttribute>) -> TokenStream2 {
    // Is it a struct?
    if let syn::Data::Struct(DataStruct { .. }) = ast.data {
        let generics = &ast.generics;
        let (_, ty_generics, _) = generics.split_for_impl();

        let name = &ast.ident;

        // generated methods
        let identifier_impl = produce_identifier_impl(&ast.ident);
        let default_impl = produce_default_impl(ast, &attrs);
        let ui_impl = produce_ui_impl(ast, &attrs);
        let enabled_impl = produce_enabled();
        let produce_as_any_impl = produce_as_any_impl();

        let muu = quote! {
            use std::ops::Deref;
            use yew::{html, Html, Callback, Context};
            use rust_internal::ui::textbox::TextBox;
            use rust_internal::ui::{numberbox::NumberBox, checkbox::Checkbox};
            use crate::ui::app::App;
            use crate::ui::app::EditorMessages;
            use std::any::Any;

            impl<Data, Modes> crate::PluginWithOptions<Data, Modes> for #name #ty_generics where
                Data: Renderer + Default + 'static,
                Modes: Clone + std::cmp::PartialEq + Eq + std::hash::Hash + 'static, {

                #identifier_impl
                #enabled_impl
                #ui_impl
            }

            impl<Data> crate::plugins::plugin::AnyPlugin<Data> for #name #ty_generics where
                Data: Renderer + Default + 'static, {

                #produce_as_any_impl

            }

            #default_impl
        };

        muu
    } else {
        quote! {}
    }
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
