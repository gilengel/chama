use std::ops::Deref;

use wasm_bindgen::JsCast;
use web_sys::{console, EventTarget, HtmlInputElement, KeyboardEvent};
use yew::events::Event;
use yew::{function_component, html, use_state, Callback, Html, use_context, classes, Children};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct DialogProps {
    pub children: Children,
}

#[function_component]
pub fn Dialog(props: &DialogProps) -> Html {
    html! {
        <div class="dialog">
            <button class="close">
                <span class="material-icons">{"close"}</span>
            </button>
            
            { for props.children.iter() } 
        </div>
    }
}
