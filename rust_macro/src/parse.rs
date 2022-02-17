use crate::{Attribute, HiddenAttribute, VisibleAttribute};
use crate::get_mandatory_meta_value;
use crate::attribute_type;
use crate::PluginAttribute;

use proc_macro_error::abort;

use syn::{DeriveInput, Meta, Data, Fields};
use syn::spanned::Spanned;

pub(crate) fn parse_attr(attr: &syn::Attribute) -> (Vec<Meta>, bool) {
    use syn::{punctuated::Punctuated, Token};

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