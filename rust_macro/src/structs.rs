use syn::{Ident, Lit, Meta};

use proc_macro2::TokenStream as TokenStream2;

/// Represents the attribute data that will added to the ui
pub(crate) struct VisibleAttribute {
    /// Name is equivalent to the variable name
    pub name: Ident,

    /// Short label that will displayed before the input
    pub label: Lit,

    /// Longer description to help the user to understand what the attribute represents
    pub description: Lit,
}

pub(crate) struct HiddenAttribute {
    pub name: Ident,
}

pub(crate) enum Attribute {
    Visible(VisibleAttribute),
    Hidden(HiddenAttribute),
}

pub(crate) type PluginAttribute = (Attribute, TokenStream2, Vec<Meta>);

#[derive(Debug)]
pub(crate) struct GenericParam {
    pub ty: Ident,
    pub execution_behaviour: Ident,
}

#[derive(Debug, PartialEq)]
pub(crate) enum EditorPluginArg {
    Skip,
    SpecificTo(Ident),
    ExecutionBehaviour(Ident),
}

#[derive(Debug)]
pub(crate) struct EditorPluginArgs {
    pub args: Vec<EditorPluginArg>,
}
