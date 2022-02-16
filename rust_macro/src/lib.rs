

extern crate proc_macro;

use colored::Colorize;
use generate::generate_option_element;
use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree, Punct, Spacing, Group, };
use proc_macro2::{Ident, Span};
use proc_macro2::Delimiter;
use proc_macro_error::{abort, proc_macro_error, ResultExt};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type, Lit};


extern crate syn;
extern crate quote;
extern crate proc_macro2;

use syn::{DataStruct, Meta};

mod generate;


fn attribute_type(ty: Type) -> TokenStream2 {
    match ty {
        Type::Path(path) => {
            path.into_token_stream()     
        },
        _ => todo!()
    }
}


type PluginAttribute = (Attribute, TokenStream2, Vec<Meta>);

pub(crate) struct Attribute {
    pub name: Ident,
    pub label: Lit,
    pub description: Lit,
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn editor_plugin(_args: TokenStream, input: TokenStream) -> TokenStream  {
    let mut ast = parse_macro_input!(input as DeriveInput);
    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {           
            match &mut struct_data.fields {
                syn::Fields::Named(fields) => {
                    fields
                        .named
                        .push(syn::Field::parse_named.parse2(quote! { __enabled: bool }).unwrap());
                }   
                _ => {
                    ()
                }
            }    
            
            return quote! {
                use rust_macro::PluginWithOptions;

                #[derive(PluginWithOptions)]
                #ast
            }.into();
        }
        _ => abort!(ast, "`add_field` has to be used with structs "),
    }
}

#[proc_macro_error]
#[proc_macro_derive(PluginWithOptions, attributes(get, with_prefix, option))]
pub fn plugin_with_options(input: TokenStream) -> TokenStream {
    
    // Parse the string representation
    let mut ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for plugin");

    let mut attrs: Vec<PluginAttribute> = Vec::new();
    match &mut ast.data {
        Data::Struct(s) => {
            match &mut s.fields {
                Fields::Named(n) => {
                    
                    for named in &mut n.named { // Field
                        let name = named.ident.as_ref().unwrap();
                        let ty = attribute_type(named.ty.clone());
                        
                        for attribute in &named.attrs {                            
                            if !attribute.path.is_ident("option") {
                                panic!("attribute {} has no option annotation.", name.to_string());
                            }

                            let metas = parse_attr(attribute);
                            let label = get_mandatory_meta_value(&metas, "label").unwrap_or_else(|| abort!(name, "the attribute {} is missing for {}", "label".red(), name));
                            let description = get_mandatory_meta_value(&metas, "description").unwrap_or_else(|| abort!(name, "the attribute {} is missing for {}", "description".red(), name));

                            attrs.push((Attribute { name: name.clone(), label: label.clone(), description: description.clone() }, ty.clone(), metas.clone()));
                        }                        
                    }
                    n.named.clear();
                },
                _ => panic!("Only works on named attributes")
            }
        },
        _ => panic!("Derive macro \"Plugin\" can only applied to a structs. Use it like this:\n\n#[derive(Plugin)]\nstruct MyPlugin{{}};")
    }

    for attr in &ast.attrs {
        panic!("{:?}", attr);
    }

    // Build the impl
    let gen = produce(&ast, attrs);
    


    // Return the generated impl
    gen.into()
}

pub(crate) fn get_mandatory_meta_value<'a>(meta_attrs: &'a Vec<Meta>, identifier: &str) -> Option<&'a Lit> {
    if let Some(default_meta) = meta_attrs.iter().find(|meta| {
        meta.path().is_ident(identifier)
    }) {
        if let Meta::NameValue(e) = &default_meta {
            return Some(&e.lit);
        }
    }
    
    None
}

fn parse_attr(attr: &syn::Attribute) -> Vec<Meta> {
    use syn::{punctuated::Punctuated, Token};

    if attr.path.is_ident("option") {
        let last= attr
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .unwrap_or_abort()
            .into_iter()
            .inspect(|meta| {
                if !(meta.path().is_ident("default")
                    || meta.path().is_ident("min")
                    || meta.path().is_ident("max")
                    || meta.path().is_ident("label")
                    || meta.path().is_ident("description"))
                {
                    abort!(meta.path().span(), "unknown parameter")
                }
            })
            .fold(vec![], |mut last, meta| {               
                last.push(meta);
                
                last
            });

        return last;
    }

    vec![]
}

pub(crate) fn callback(callback_name: &Ident, ty: &TokenStream2) -> TokenStream2 {
    quote!{                
        let #callback_name : Callback<(&'static str, &'static str, #ty)> = ctx.link().callback(|(plugin, attribute, value)| {                
            EditorMessages::PluginOptionUpdated((plugin, attribute, Box::new(value)))
        });
        
    }
}

fn checkbox(plugin: &str, attribute: &str, default_value: bool) -> (TokenStream2, TokenStream2) {
    let callback_name = format_ident!("cb_{}", attribute);

    let callback = quote!{ 
        let #callback_name : Callback<(&'static str, &'static str, bool)> = ctx.link().callback(|(plugin, attribute, value)| {                
            EditorMessages::PluginOptionUpdated((plugin, #attribute, Box::new(value)))
        });
    };

    let element = quote! {
        <Checkbox 
            plugin={#plugin} 
            attribute="__enabled" 
            value={#default_value} 
            on_value_change={#callback_name} 
        />
    };

    (element, callback)
}

fn produce(ast: &DeriveInput, attrs: Vec<PluginAttribute>) -> TokenStream2 {
    let name = &ast.ident;
    let name_str = name.to_string();
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();


    // Is it a struct?
    if let syn::Data::Struct(DataStruct { .. }) = ast.data {
        let mut t = TokenStream2::new();
        t.extend(vec![
            TokenTree::Ident(Ident::new("html",  Span::call_site())),             
            TokenTree::Punct(Punct::new('!', Spacing::Alone))
        ]);

        let mut callbacks: Vec<TokenStream2> = vec![];
        
        
        let plugin = name_str.clone();
        
        let mut inner = TokenStream2::new();

        // match arms used when a plugin received a message one of its properties was changed. 
        // The match arm maps the message content to the corresponding property
        let mut arms: Vec<TokenStream2> = vec![];

        // Used for the default impl function
        let mut defaults: Vec<TokenStream2> = vec![];

        // List of the html elements
        let mut elements: Vec<TokenStream2> = vec![
            quote! { <div> },
            quote! { <div><h2>{#name_str}</h2> },
        ];

        // Enable checkbox for each plugin
        let enabled_checkbox = checkbox(&plugin.clone(), "__enabled", true);
        elements.push(enabled_checkbox.0);
        elements.push( quote! { </div> } );
        callbacks.push(enabled_checkbox.1);    
        defaults.push(quote!{ __enabled: true }); 
        
        arms.push(quote! { "__enabled" => { if let Some(value) = value.as_ref().downcast_ref::<bool>() { self.__enabled = *value } }});

        // Now for all attributes defined be the plugin developer
        for (attr, ty, metas) in attrs {
            let attr = generate_option_element(&plugin, attr, ty, metas);
            elements.push(attr.element);
            callbacks.push(attr.callback);
            arms.push(attr.arm);
            defaults.push(attr.default);
        }

        elements.push(quote!{</div>});
        inner.append_all(elements);
        t.extend(vec![TokenTree::Group(Group::new(Delimiter::Brace, inner))]);       


        let gen = quote! {        
            use std::ops::Deref;   
            use yew::{html, Html, Callback, Context};
            use rust_internal::ui::textbox::TextBox;
            use rust_internal::ui::{numberbox::NumberBox, checkbox::Checkbox};
            use crate::ui::app::App;
            use crate::ui::app::EditorMessages;
            use std::any::Any;

            impl<Data, Modes> PluginWithOptions<Data, Modes> for #name #ty_generics where 
                Data: Renderer + Default + 'static,
                Modes: Clone + std::cmp::PartialEq + Eq + std::hash::Hash + 'static, {        
                
                fn identifier() -> &'static str where Self: Sized {
                    #name_str
                }
                
                fn view_options(&self, ctx: &Context<App<Data, Modes>>) -> Html {
                    #(#callbacks)*
                    #t                    
                }

                fn update_property(&mut self, property: &str, value: Box<dyn Any>) {    
                    web_sys::console::log_1(&format!("from plugin property {} {:?}", property, value).into());

                    match property {
                        #(#arms),*
                        _ => { web_sys::console::log_1(&":(".into()); }
                    }
                    
                }

                fn enabled(&self) -> bool {
                    self.__enabled
                }

                fn as_any(&self) -> &dyn std::any::Any { self }
                fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
            }    
            
            impl #impl_generics Default for #name #ty_generics #where_clause{
                fn default() -> Self {
                    Self {
                        #(#defaults),*
                    }
                }
            }
            
        };        

        gen

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
