use yew::{function_component, html, Html, Children};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct RibbonTabProps
{
    pub label: String,
    pub children: Children
}

#[function_component]
pub fn RibbonTab(props: &RibbonTabProps) -> Html
{

    html! {
        <div class="ribbon_tab">
            { for props.children.iter() } 
        </div>
    }
}
