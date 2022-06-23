use yew::{function_component, html, Html, Children};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct ToolbarProps
{
    pub children: Children
}

#[function_component]
pub fn Toolbar(props: &ToolbarProps) -> Html
{

    html! {
        <ul class="toolbar">
        { for props.children.iter() } 
        </ul>
    }
}
