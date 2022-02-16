use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::Meta;

use crate::{callback, get_mandatory_meta_value, Attribute};

pub(crate) struct PluginOptionElement {
    pub element: TokenStream2,
    pub callback: TokenStream2,
    pub arm: TokenStream2,
    pub default: TokenStream2,
}

pub(crate) fn generate_option_element(
    plugin: &String,
    attr: Attribute,
    ty: TokenStream2,
    metas: Vec<Meta>,
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
    let ty: proc_macro2::TokenStream = ty.parse().unwrap();

    let default = match get_mandatory_meta_value(&metas, "default") {
        Some(e) => quote! { #e },
        None => quote! { #ty::default()},
    };

    let attribute = attr.name.to_string();
    let attr_ident = format_ident!("{}", attr.name);

    // Add the callback to inform the editor that the option has been updated by the user
    let callback_name = format_ident!("cb_{}", attr.name);
    result.callback = callback(&callback_name, &ty);

    result.arm = quote! {
                #attribute => { if let Some(value) = value.as_ref().downcast_ref::<#ty>() { self.#attr_ident = *value } }
    };

    result.default = quote! {
        #attr_ident: #default
    };

    let label = attr.label;
    let description = attr.description;

    let number_types: Vec<&str> = vec![
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64",
    ];
    if number_types.contains(&&ty.to_string()[..]) {
        let min = get_mandatory_meta_value(&metas, "min")
            .unwrap_or_else(|| panic!("the attribute {} is missing for {}", "min", attribute));
        let max = get_mandatory_meta_value(&metas, "max")
            .unwrap_or_else(|| panic!("the attribute {} is missing for {}", "max", attribute));

        result.element = quote! {
        <div>
            <label>{#label}</label>
            <NumberBox<#ty>
                plugin={#plugin}
                attribute={#attribute}
                min={#min}
                max={#max}
                value={#default}
                on_value_change={#callback_name}
            />
            <label class="description">{#description}</label>
        </div>};
    } else if ty.to_string() == "bool" {
        result.element =  quote! {
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

    result
}
