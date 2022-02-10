

use std::any::Any;
use yew::Html;

pub trait GetPlugin<T> {
    fn get<S>(&self) -> Option<S>;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

pub struct Attribute {
    pub label: String,
    pub description: String
}
pub struct NumberAttribute {
    pub default: i128,
    pub min: i128,
    pub max: i128,
}

pub struct TextAttribute {

}

pub enum PluginAttributes {
    Number((Attribute, NumberAttribute)),
    Text((Attribute, TextAttribute))
}

pub trait PluginOptions {
    fn view_options(&self) -> Html;
}

