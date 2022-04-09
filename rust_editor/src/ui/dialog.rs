use yew::{function_component, html, Html, Children};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct DialogProps {
    pub children: Children,
}

#[function_component]
pub fn Dialog(props: &DialogProps) -> Html {
    html! {
        <div class="dialog">
            { for props.children.iter() } 
        </div>
    }
}
