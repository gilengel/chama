use yew::{function_component, html, Html, Children};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct RibbonTabGroupProps
{
    pub children: Children,
    pub title: String
}

#[function_component]
pub fn RibbonTabGroup(props: &RibbonTabGroupProps) -> Html
{

    html! {
        <div class="ribbon_tab_group">
            <div class="content">
                { for props.children.iter() } 
            </div>
            <span>{&props.title}</span>
        </div>
    }
}
