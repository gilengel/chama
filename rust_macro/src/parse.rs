use proc_macro_error::abort;

use syn::parse::{ParseStream, Parse};
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Meta, Data, Fields, Token, Error, Lit};
use syn::spanned::Spanned;



use crate::attribute_type;
use crate::structs::{Attribute, VisibleAttribute, HiddenAttribute, GenericParam, EditorPluginArg, EditorPluginArgs, PluginAttribute};

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

pub(crate) fn parse_attrs(ast: &DeriveInput) -> Vec<PluginAttribute>{
    let mut attrs: Vec<PluginAttribute> = Vec::new();
    
    match &ast.data {
        Data::Struct(s) => {
            match &s.fields {
                Fields::Named(n) => {
                    
                    for named in &n.named { // Field
                        let name = named.ident.as_ref().unwrap();
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

impl Parse for EditorPluginArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) {
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

            Err(input.error(format!("unknown identifier got {}. Allowed identifiers are \"skip\", \"specific_to=Type\" and \"execution=PluginExecutionBehaviour\"", ident)))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for EditorPluginArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parsed_args: Punctuated<EditorPluginArg, syn::Token![,]> =
            input.parse_terminated(EditorPluginArg::parse)?;

        // There must be a better way to do this but I will leave it for now like this 2022-02-18
        let mut args = vec![];
        for i in parsed_args {
            args.push(i);
        }
        Ok(EditorPluginArgs { args })
    }
}