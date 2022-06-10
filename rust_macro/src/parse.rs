

use proc_macro2::Span;
use proc_macro_error::abort;

use syn::parse::{ParseStream, Parse};
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Meta, Data, Fields, Token, Error, Lit};
use syn::spanned::Spanned;

use ansi_term::Color::Cyan;

use crate::attribute_type;
use crate::structs::{Attribute, VisibleAttribute, HiddenAttribute, GenericParam, EditorPluginArg, EditorPluginArgs, PluginAttribute};

/// Parses a single attribute. Depending on the attribute type different tags are necessary. 
/// The function does not check if necessary tags are present. Instead, it only checks if the defined
/// tags are valid.
/// 
/// ```
/// #[editor_plugin(specific_to=Map)]
/// pub struct Sync {
///     
///     #[option(
///         label = "URL",
///         description = ""
///     )]
///     url: String,
/// }
/// ```
/// 
/// Be aware that the attribute does not supports all built-in Rust types and might not support your
/// custom struct type. So far the following types are supported:
/// 
/// * i8
/// * i16
/// * i32
/// * i64
/// * i128
/// * isize
/// * u8
/// * u16
/// * u32
/// * u64
/// * u128
/// * usize
/// * f32
/// * f64
/// * String
/// * Vec
/// 
pub(crate) fn parse_attr(attr: &syn::Attribute) -> (Vec<Meta>, bool) {
    if attr.path.is_ident("option") {
        let (skip, metas)= attr
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .unwrap_or_else(|_| abort!(attr, "no parameters defined"))
            .into_iter()
            .inspect(|meta| {
                if !(meta.path().is_ident("default")
                    || meta.path().is_ident("min")
                    || meta.path().is_ident("max")
                    || meta.path().is_ident("label")
                    || meta.path().is_ident("description")
                    || meta.path().is_ident("skip"))
                {
                    abort!(meta.path().span(), "unknown parameter");
                }
            })
            .fold((false, Vec::<Meta>::new()), |(skip, mut metas), meta| {  
                if meta.path().is_ident("skip") {
                    (true, metas)
                } else {
                    metas.push(meta);                
                    (skip, metas)
                }       

                
            });

        return (metas, skip);
    }

    (vec![], false)
}

/// Parses all attributes for a plugin
/// Attributes are specified as an annotation to the member variable and start with the keyword `option`.
/// Each attribute needs a label and a description see the following example how to specify them.
///
/// ```
/// #[editor_plugin(specific_to=Map)]
/// pub struct Sync {
///     
///     #[option(
///         label = "URL",
///         description = ""
///     )]
///     url: String,
/// }
/// ```
/// 
pub(crate) fn parse_attrs(ast: &DeriveInput) -> Vec<PluginAttribute>{
    let mut attrs: Vec<PluginAttribute> = Vec::new();
    
    match &ast.data {
        Data::Struct(s) => {
            match &s.fields {
                Fields::Named(n) => {
                    
                    for named in &n.named { // Field
                        let name = named.ident.as_ref().unwrap_or_else(|| abort!(named, "MUU"));
                        let ty = attribute_type(named.ty.clone());
                        
                        for attribute in &named.attrs {                            
                            if !attribute.path.is_ident("option") {
                                panic!("attribute {} has no option annotation.", name.to_string());
                            }

                            let ( metas, hidden) = parse_attr(attribute);
                                
                            if !hidden {
                                let label = get_mandatory_meta_value(&metas, "label").unwrap_or_else(|| abort!(name, "the attribute {} is missing for {}", "label", name));
                                let description = get_mandatory_meta_value(&metas, "description").unwrap_or_else(|| abort!(name, "the attribute {} is missing for {}", "description", name));

                                attrs.push((Attribute::Visible(VisibleAttribute { name: name.clone(), label: label.clone(), description: description.clone() }), ty.clone(), metas.clone()));
                            } else {
                                attrs.push((Attribute::Hidden(HiddenAttribute { name: name.clone() }), ty.clone(), metas.clone()));
                            }                              
                        }                        
                    }
                },
                _ => panic!("Only works on named attributes")
            }
        },
        _ => panic!("Derive macro \"Plugin\" can only applied to a structs. Use it like this:\n\n#[derive(Plugin)]\nstruct MyPlugin{{}};")
    }

    attrs
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

impl Parse for GenericParam {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let content;
        syn::parenthesized!(content in input);
        let ty = content.parse()?;
        content.parse::<Token![,]>()?;
        let execution_behaviour = content.parse()?;        
        
        Ok(GenericParam { ty, execution_behaviour })
    }
}

mod kw {
    syn::custom_keyword!(skip);
    syn::custom_keyword!(specific_to);
    syn::custom_keyword!(execution);
    syn::custom_keyword!(description);
}

impl Parse for EditorPluginArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        //let lookahead = input.lookahead1();
        
        
        if input.peek(kw::skip) ||
        input.peek(kw::specific_to) ||
        input.peek(kw::execution) ||
        input.peek(kw::description)
        {
            let ident = input.parse::<syn::Ident>()?;

            if ident == "skip" {
                return Ok(EditorPluginArg::Skip);
            }

            if ident == "specific_to" {
                input.parse::<syn::Token![=]>()?;
                let ty = input.parse::<syn::Ident>()?;

                return Ok(EditorPluginArg::SpecificTo(ty));
            }

            if ident == "execution" {
                input.parse::<syn::Token![=]>()?;
                let ty = input.parse::<syn::Ident>()?;

                //let behaviour = PluginExecutionBehaviour::from_str(&ty.to_string()).unwrap();
                return Ok(EditorPluginArg::ExecutionBehaviour(ty));
            }

            // description
            input.parse::<syn::Token![=]>()?;
            let ty = input.parse::<syn::Expr>()?;

            return Ok(EditorPluginArg::Description(ty));
            
        }
        else {
            Err(input.error(format!("failed to parse plugin: use of undefined tag.\n\n{}: Plugins support the following tags:\n - skip\n - specific_to\n - execution\n - shortkey", Cyan.paint("help"))))
        }

    }
}

impl Parse for EditorPluginArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parsed_args: Punctuated<EditorPluginArg, syn::Token![,]> =
            input.parse_terminated(EditorPluginArg::parse)?;

        let args : Vec<EditorPluginArg> = parsed_args.into_iter().collect();
        Ok(EditorPluginArgs { args })
    }
}