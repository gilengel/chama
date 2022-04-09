use yew::{function_component, html, Html, Children};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct PanelProps {
    pub children: Children,
}

#[function_component]
pub fn Panel(props: &PanelProps) -> Html {
    html! {
        <div class="panel">
            { for props.children.iter() } 
        </div>
    }
}
