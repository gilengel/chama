use std::ops::Deref;

use yew::{function_component, html, use_state, Html, classes};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct TextBoxProps {
}

#[function_component]
pub fn TextBox(_props: &TextBoxProps) -> Html {


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
