use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::TokenStreamExt;
use quote::{format_ident, quote};
use std::str::FromStr;
use syn::{DataStruct, DeriveInput, Ident, Meta};

use crate::GenericParam;

use proc_macro2::Delimiter;
use proc_macro2::Span;
use proc_macro2::{Group, Punct, Spacing, TokenTree};

use crate::structs::Attribute;
use crate::structs::PluginAttribute;
use crate::{parse::get_mandatory_meta_value, structs::VisibleAttribute};

pub(crate) struct PluginOptionElement {
    pub element: TokenStream2,
    pub callback: TokenStream2,
    pub arm: TokenStream2,
    pub default: TokenStream2,
}

pub(crate) fn generate_default_arm(
    attr_ident: &Ident,
    ty: &TokenStream2,
    metas: &Vec<Meta>,
) -> TokenStream2 {
    // I guess there is a smarter way to do it but converting it to a str, replace < with :: < and converting it back
    // works and handles the case if the user defines a variable with generic type like
    // position: Coordinate<f64> instead of position: Coordinate::<f64>
    let ty = TokenStream2::from_str(&str::replace(&ty.clone().to_string(), " <", ":: <")[..])
        .unwrap_or_else(|_| abort!(ty, "BLOWFISH"));

    let default = match get_mandatory_meta_value(&metas, "default") {
        Some(e) => quote! { #e },
        None => quote! { #ty::default()},
    };

    quote! {
        #attr_ident: #default
    }
}

fn callback(callback_name: &Ident, ty: &TokenStream2) -> TokenStream2 {
    quote! {
        let #callback_name : Callback<(&'static str, &'static str, #ty)> = ctx.link().callback(|(plugin, attribute, value)| {
            EditorMessages::PluginOptionUpdated((plugin, attribute, Box::new(value)))
        });

    }
}

pub(crate) fn checkbox(
    plugin: &str,
    attribute: &str,
    default_value: bool,
) -> (TokenStream2, TokenStream2) {
    let callback_name = format_ident!("cb_{}", attribute);

    let callback = quote! {
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

pub(crate) fn generate_option_element(
    plugin: &String,
    attr: &VisibleAttribute,
    ty: &TokenStream2,
    metas: &Vec<Meta>,
) -> PluginOptionElement {
    let mut result = PluginOptionElement {
        element: quote! {},
        callback: quote! {},
        arm: quote! {},
        default: quote! {},
    };

    // Allow the user to define generic datatypes as usuall like Coordinate<f64>. Without this replacing we would force user
    // to write instead Coordinate::<f64> which is odd.
    let ty = &str::replace(&ty.clone().to_string(), " <", "::<");
    let ty: proc_macro2::TokenStream = ty.parse().unwrap_or_else(|_| abort!(ty, "BLOWFISH 2"));

    let attribute = attr.name.to_string();
    let attr_ident = format_ident!("{}", attr.name);

    // Add the callback to inform the editor that the option has been updated by the user
    let callback_name = format_ident!("cb_{}", attr.name);
    result.callback = callback(&callback_name, &ty);

    result.arm = quote! {
        #attribute => { if let Some(value) = value.as_ref().downcast_ref::<#ty>() { self.#attr_ident = *value; } }
    };

    let default = generate_default_arm(&attr_ident, &ty, &metas);

    let label = attr.label.clone();
    //let description = attr.description;

    let number_types: Vec<&str> = vec![
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64",
    ];
    if number_types.contains(&&ty.to_string()[..]) {
        let min = get_mandatory_meta_value(&metas, "min").unwrap_or_else(|| {
            abort!(
                attr.name,
                format!("the attribute {} is missing for {}", "min", attribute)
            )
        });
        let max = get_mandatory_meta_value(&metas, "max").unwrap_or_else(|| {
            abort!(
                attr.name,
                format!("the attribute {} is missing for {}", "max", attribute)
            )
        });
        let value = get_mandatory_meta_value(&metas, "default").unwrap_or_else(|| {
            abort!(
                attr.name,
                format!("the attribute {} is missing for {}", "default", attribute)
            )
        });

        result.element = quote! {
        <div>
            <label>{#label}</label>
            <NumberBox<#ty>
                plugin={#plugin}
                attribute={#attribute}
                min={#min}
                max={#max}
                value={#value}
                on_value_change={#callback_name}
            />

            /*<label class="description">{#description}</label>*/
        </div>};
    } else if ty.to_string() == "bool" {
        result.element = quote! {
        <div>
            <label>{#label}</label>
            <Checkbox
                plugin={#plugin}
                attribute={#attribute}
                value={true}
                on_value_change={#callback_name}
            />
        </div>};
    }

    result.default = default;
    result
}

fn produce_default_impl(
    ast: &DeriveInput,
    attrs: &Vec<PluginAttribute>,
    execution_behaviour: &Ident,
) -> TokenStream2 {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Used for the default impl function
    let mut defaults: Vec<TokenStream2> = vec![];
    defaults.push(quote! { __enabled: std::rc::Rc::new(RefCell::new(true)) });
    defaults.push(quote! { __execution_behaviour: rust_internal::PluginExecutionBehaviour::#execution_behaviour });

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
           *self.__enabled.as_ref().borrow()
        }

        fn enable(&mut self) {
            self.__enabled.replace(true);
        }

        fn disable(&mut self) {
            self.__enabled.replace(false);
        }
    }
}

fn produce_as_any_impl() -> TokenStream2 {
    quote! {
        fn as_any(&self) -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    }
}

fn produce_ui_impl(
    ast: &DeriveInput,
    attrs: &Vec<PluginAttribute>,
    generic_type: &Ident,
) -> TokenStream2 {
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

    arms.push(quote! { "__enabled" => { if let Some(value) = value.as_ref().downcast_ref::<bool>() { *self.__enabled.borrow_mut() = *value } }});

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
        fn view_options(&self, ctx: &Context<App<#generic_type>>) -> Html {
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

fn produce_use_statements(crate_name: &Ident) -> TokenStream2 {
    quote! {
        use std::ops::Deref;
        use yew::{html, Html, Callback, Context};
        use rust_internal::ui::textbox::TextBox;
        use rust_internal::ui::{numberbox::NumberBox, checkbox::Checkbox};
        use #crate_name::ui::app::App;
        use #crate_name::ui::app::EditorMessages;
        use #crate_name::plugins::plugin::SpecialKey;
        use std::any::Any;
        use std::borrow::Borrow;
        use std::cell::RefCell;
        use std::rc::Rc;
    }
}

fn crate_name() -> Ident {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let crate_name = Ident::new(
        if &crate_name[..] == "rust_editor" {
            "crate"
        } else {
            "rust_editor"
        },
        Span::call_site(),
    );

    crate_name
}

pub(crate) fn produce(
    ast: &DeriveInput,
    attrs: Vec<PluginAttribute>,
    param: &GenericParam,
    skip_ui_gen: bool,
) -> TokenStream2 {
    let (where_clause_plugins_with_options, where_clause_any_plugin) = if param.ty == "Data" {
        (
            quote! { where Data: Renderer + Default + 'static },
            quote! { where Data: Renderer + Default + 'static },
        )
    } else {
        (quote! {}, quote! {})
    };

    let (impl_plugins_with_options, impl_any_plugin) = if param.ty == "Data" {
        (quote! { impl<Data> }, quote! { impl<Data> })
    } else {
        (quote! { impl }, quote! { impl })
    };

    // Is it a struct?
    if let syn::Data::Struct(DataStruct { .. }) = ast.data {
        let generics = &ast.generics;
        let (_, ty_generics, _) = generics.split_for_impl();

        let name = &ast.ident;

        // generated methods
        let identifier_impl = produce_identifier_impl(&ast.ident);
        let default_impl = produce_default_impl(ast, &attrs, &param.execution_behaviour);
        let ui_impl = match skip_ui_gen {
            true => quote! {},
            false => produce_ui_impl(ast, &attrs, &param.ty),
        };
        let enabled_impl = produce_enabled();
        let produce_as_any_impl = produce_as_any_impl();

        let crate_name = crate_name();

        let generic_type = param.ty.clone();

        let use_statements = produce_use_statements(&crate_name);
        let muu = quote! {
            #use_statements

            #impl_plugins_with_options #crate_name::plugins::plugin::PluginWithOptions<#generic_type> for #name #ty_generics #where_clause_plugins_with_options
            {
                #identifier_impl
                #enabled_impl
                #ui_impl

                fn execution_behaviour(&self) -> &rust_internal::PluginExecutionBehaviour {
                    &self.__execution_behaviour
                }
            }


            #impl_any_plugin #crate_name::plugins::plugin::AnyPlugin<#generic_type> for #name #ty_generics #where_clause_any_plugin
            {
                #produce_as_any_impl
            }

            #default_impl
        };

        muu
    } else {
        quote! {}
    }
}
