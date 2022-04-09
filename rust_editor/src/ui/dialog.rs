use yew::{function_component, html, Html, Children};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct DialogProps {
    pub title: String,
    pub children: Children,
}

#[function_component]
pub fn Dialog(props: &DialogProps) -> Html {
    html! {
        <div class="overlay">
        <div class="dialog">
            <h6>{props.title.clone()}</h6>

            <div class="controls">
            { for props.children.iter() } 
            </div>
        </div>
        </div>
    }
}
