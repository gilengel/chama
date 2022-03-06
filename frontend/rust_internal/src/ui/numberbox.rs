use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

use std::fmt::Debug;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::{classes, function_component, html, use_state, Callback, Html, InputEvent};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct NumberBoxProps<T>
where
    T: PartialEq + Default,
{
    pub plugin: &'static str,
    pub attribute: &'static str,

    pub min: T,
    pub max: T,

    #[prop_or_default]
    pub default: T,

    pub value: T,
    pub on_value_change: Callback<(&'static str, &'static str, T)>,
}

#[function_component]
pub fn NumberBox<T>(props: &NumberBoxProps<T>) -> Html
where
    T: std::str::FromStr + PartialOrd + Display + Default + Copy + 'static,
    <T as FromStr>::Err: Debug,
{
    let value_handle = use_state(|| props.value.to_string());
    let value = value_handle.deref().clone();
    let range_handle = use_state(|| (props.min, props.max));

    let error_handle = use_state(|| true);
    let error = error_handle.deref().clone();

    let callback = props.on_value_change.clone();

    let plugin = props.plugin;
    let attribute = props.attribute;

    let oninput = Callback::from(move |e: InputEvent| {
        let target: EventTarget = e
            .target()
            .expect("Event should have a target when dispatched");

        let value = target.unchecked_into::<HtmlInputElement>().value();

        let range = *range_handle;
        match value.clone().parse::<T>() {
            Ok(value) => error_handle.set(!(value >= range.0 && value <= range.1)),
            Err(_) => error_handle.set(true),
        }

        value_handle.set(value.clone());

        match value.parse::<T>() {
            Ok(e) => {
                callback.emit((plugin, attribute, e));
            }
            Err(_) => {}
        }
    });

    html! {
        <div class="textbox">
            <input class={classes!(error.then(|| Some("error")))} type="text" {oninput} value={value} />

            if error {
                <label class="info">{format!{"Only values from {} till {} are valid", props.min, props.max}}</label>
            }
        </div>
    }
}
