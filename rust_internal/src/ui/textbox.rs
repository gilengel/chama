use std::ops::Deref;

use wasm_bindgen::JsCast;
use web_sys::{console, EventTarget, HtmlInputElement, KeyboardEvent};
use yew::events::Event;
use yew::{function_component, html, use_state, Callback, Html, use_context, classes};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct TextBoxProps {
}

#[function_component]
pub fn TextBox(props: &TextBoxProps) -> Html {


    let error_handle = use_state(|| false);
    let error = error_handle.deref().clone();

    html! {
        <div class="textbox">
            <input class={classes!(error.then(|| Some("error")))} type="text" />

            if error {
                <label class="info">{format!{"Some Error"}}</label>
            }
        </div>
    }
}
