use std::cell::RefCell;
use std::fmt::Display;
use std::ops::Deref;
use std::rc::Rc;
use std::str::FromStr;

use std::fmt::Debug;
use wasm_bindgen::JsCast;
use web_sys::{console, EventTarget, HtmlInputElement, KeyboardEvent};
use yew::events::Event;
use yew::{classes, function_component, html, use_context, use_mut_ref, use_state, Callback, Html, InputEvent};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct CheckboxProps
{
    pub plugin: &'static str,
    pub attribute: &'static str,
    pub value: bool,
    pub on_value_change: Callback<(&'static str, &'static str, bool)>,
}

#[function_component]
pub fn Checkbox(props: &CheckboxProps) -> Html
{
    let value_handle = use_state(|| props.value);
    let value = value_handle.deref().clone();

    let error_handle = use_state(|| false);
    let error = error_handle.deref().clone();

    let callback = props.on_value_change.clone();

    let plugin = props.plugin;
    let attribute = props.attribute;

    let oninput = Callback::from(move |e: InputEvent| {
        
        let target: EventTarget = e
            .target()
            .expect("Event should have a target when dispatched");

        let value = target.unchecked_into::<HtmlInputElement>().checked();
        value_handle.set(value.clone());  
        
        callback.emit((plugin, attribute, value));    
    });

    html! {
        <div class="checkbox">
            <input class={classes!(error.then(|| Some("error")))} type="checkbox" checked={value} {oninput} />

            if error {
                <label class="info">{format!("Some Error")}</label>
            }
        </div>
    }
}
