use yew::{function_component, html, Html, Children};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct RibbonProps
{
    pub children: Children
}

#[function_component]
pub fn Ribbon(props: &RibbonProps) -> Html
{

    html! {
        <div class="ribbon">
        { for props.children.iter() } 
        </div>
    }
}
