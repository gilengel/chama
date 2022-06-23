use std::ops::Deref;

use regex::Regex;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::{classes, function_component, html, use_state, Callback, Html, InputEvent};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct TextBoxProps
{
    pub plugin: &'static str,
    pub attribute: &'static str,

    #[prop_or_default]
    pub validation_regex: &'static str,

    pub value: String,
    pub on_value_change: Callback<(&'static str, &'static str, String)>,
}

#[function_component]
pub fn TextBox(props: &TextBoxProps) -> Html
{
    
    let value_handle = use_state(|| props.value.to_string());
    let value = value_handle.deref().clone();

    let error_handle = use_state(|| false);
    let error = error_handle.deref().clone();

    let callback = props.on_value_change.clone();

    let plugin = props.plugin;
    let attribute = props.attribute;

    let re = Regex::new(props.validation_regex).unwrap();
    let oninput = Callback::from(move |e: InputEvent| {
        let target: EventTarget = e
            .target()
            .expect("Event should have a target when dispatched");

        let value = target.unchecked_into::<HtmlInputElement>().value();

        error_handle.set(!(re.is_match(&value)));

        value_handle.set(value.clone());
        callback.emit((plugin, attribute, value));
    });
    

    html! {
        <div class="textbox">
            <input class={classes!(error.then(|| Some("error")))} type="text" {oninput} value={value} />

            if error {
                <label class="info">{format!{"Invalid value"}}</label>
            }
        </div>
    }
}
