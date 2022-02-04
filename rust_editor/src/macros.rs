use std::sync::Mutex;

use js_sys::Array;
use rust_internal::plugin::Plugin;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{Document, HtmlElement, HtmlInputElement};

use crate::{editor::Editor, plugins::camera::Renderer};

pub fn get_element(document: &Document, id: Option<String>) -> HtmlElement {
    match id {
        Some(id) => document
            .get_element_by_id(&id)
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap(),
        None => document.body().unwrap(),
    }
}

pub fn create_element(tag: &str) -> HtmlElement {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    document
        .create_element(tag)
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap()
}

pub fn insert_element(stack: &mut Vec<HtmlElement>) {
    let element = stack.pop().unwrap();
    let parent = stack.last().unwrap();

    parent.append_child(&element).unwrap();
    stack.push(element);
}

pub fn insert_classes(stack: &mut Vec<HtmlElement>, classes: &str) -> Result<(), JsValue> {
    let element = stack.last().unwrap();

    let a = Array::new();
    let classes: Vec<&str> = classes.split(',').collect();
    for class in &classes {
        a.push(&JsValue::from_str(class));
    }

    element.class_list().add(&a)?;

    Ok(())
}

pub fn set_id(stack: &mut Vec<HtmlElement>, id: &str) {
    let element = stack.last().unwrap();
    element.set_id(id);
}

pub fn set_attribute(
    stack: &mut Vec<HtmlElement>,
    attribute: &str,
    value: &str,
) -> Result<(), JsValue> {
    let element = stack.last().unwrap();
    element.set_attribute(attribute, value)?;

    Ok(())
}

pub fn set_type(stack: &mut Vec<HtmlElement>, input_type: &str) {
    let element = stack.pop().unwrap().dyn_into::<HtmlInputElement>().unwrap();
    element.set_type(input_type);

    stack.push(element.into());
}

pub fn set_onchange<F, T, M>(stack: &mut Vec<HtmlElement>, _callback: F)
where
    F: Fn(Mutex<Editor<T>>) -> () + 'static,
    T: Renderer,
    M: Plugin<T> + std::cmp::Eq + std::hash::Hash + 'static,
{
    let a = Closure::wrap(Box::new(move || {
        //callback();
    }) as Box<dyn FnMut()>);
    let element = stack.pop().unwrap();
    element.set_onclick(Some(a.as_ref().unchecked_ref()));

    // See comments in `setup_clock` above for why we use `a.forget()`.
    a.forget();

    stack.push(element);
}

pub fn set_text_content(stack: &mut Vec<HtmlElement>, content: &str) {
    let mut text = content;
    text = &text[1..text.len() - 1];
    stack.last().unwrap().set_text_content(Some(text));
}

pub fn slugify(text: &str) -> String {
    let a: String = text
        .chars()
        .map(|x| match x {
            '!' | '?' | ',' | '.' | ';' | ' ' => '-',
            _ => x,
        })
        .collect();

    a.to_ascii_lowercase()
}

#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    }
}

#[macro_export]
macro_rules! error {
    ( $( $t:tt )* ) => {
        unsafe { web_sys::console::error_1(&format!( $( $t )* ).into()) }
    }
}

#[macro_export]
macro_rules! html_impl {
    // Start of opening tag
    ($stack:ident (< $starttag:ident $($tail:tt)*)) => {
        $stack.push($crate::macros::create_element(stringify!($starttag)));
        html_impl! { $stack ($($tail)*) }
    };

    ($stack:ident (< $starttag:ident $($tail:tt)*)) => {
        $stack.push($crate::macros::create_element(stringify!($starttag)));
        html_impl! { $stack ($($tail)*) }
    };

    // PATTERN: id=""
    ($stack:ident (id = $id:literal $($tail:tt)*)) => {
        $crate::macros::set_id(&mut $stack, $id);
        html_impl! { $stack ($($tail)*) }
    };

    // PATTERN: id=""
    ($stack:ident (id = ($id:expr) $($tail:tt)*)) => {
        $crate::macros::set_id(&mut $stack, &format!("{}", $id).to_string());
        html_impl! { $stack ($($tail)*) }
    };

    // PATTERN: name=""
    ($stack:ident (name = $name:literal $($tail:tt)*)) => {
        $crate::macros::set_attribute(&mut $stack, "name", &format!("{}", $name))?;
        html_impl! { $stack ($($tail)*) }
    };


    // PATTERN: class=""
    ($stack:ident (class = $class:literal $($tail:tt)*)) => {
        $crate::macros::insert_classes(&mut $stack, $class)?;
        html_impl! { $stack ($($tail)*) }
    };

    // PATTERN: href=""
    ($stack:ident (href = $id:literal $($tail:tt)*)) => {
        html_impl! { $stack ($($tail)*) }
    };

    // PATTERN: role=""
    ($stack:ident (role = $value:literal $($tail:tt)*)) => {
        $crate::macros::set_attribute(&mut $stack, "role", $value)?;
        html_impl! { $stack ($($tail)*) }
    };

    // PATTERN: group=""
    ($stack:ident (group = $id:literal $($tail:tt)*)) => {
        $crate::macros::set_attribute(&mut $stack, "group", $value);
        html_impl! { $stack ($($tail)*) }
    };

    // PATTERN: onchange
    ($stack:ident (onchange = ($func:expr) $($tail:tt)*)) => {
        $crate::macros::set_onchange(&mut $stack, $func);
        html_impl! { $stack ($($tail)*) }
    };

    // PATTERN: attribute=value - workaround for `type` attribute
    // because `type` is a keyword in Rust
    ($stack:ident (type = $kind:literal $($tail:tt)*)) => {
        $crate::macros::set_type(&mut $stack, $kind);
        html_impl! { $stack ($($tail)*) }
    };

     // PATTERN: value
    ($stack:ident (value = ($value:expr) $($tail:tt)*)) => {
        $crate::macros::set_attribute(&mut $stack, "value", &format!("{}", $value))?;
        html_impl! { $stack ($($tail)*) }
    };

    // PATTERN: attribute=value - workaround for `type` attribute
    // because `for` is a keyword in Rust
    ($stack:ident (for = ($kind:expr) $($tail:tt)*)) => {
        $crate::macros::set_attribute(&mut $stack, "for", &format!("{}", $kind))?;
        html_impl! { $stack ($($tail)*) }
    };

    ($stack:ident ($attr:ident = $val:literal, $($tail:tt)*)) => {
        //$crate::macros::add_attribute(&mut $stack, stringify!($attr), $val);
        html_impl! { $stack ($($tail)*) }
    };

    // End of opening tag
    ($stack:ident (> $($tail:tt)*)) => {
        $crate::macros::insert_element(&mut $stack);
        html_impl! { $stack ($($tail)*) }
    };

    ($stack:ident ((for $element:ident in $collection:expr) { $($loop:tt)* } $($tail:tt)*)) => {
        for $element in $collection {
            html_impl! { $stack ($($loop)*) }
        }

        html_impl! { $stack ($($tail)*) }
    };

    // Literal
    ($stack:ident ($content:literal $($tail:tt)*)) => {
        $crate::macros::set_text_content(&mut $stack, stringify!($content));
        html_impl! { $stack ($($tail)*) }
    };

    // Expression
    ($stack:ident (($expr:expr) $($tail:tt)*)) => {
        $crate::macros::set_text_content(&mut $stack, &format!("\"{}\"", $expr));
        html_impl! { $stack ($($tail)*) }
    };

    // Self-closing of tag
    ($stack:ident (/ > $($tail:tt)*)) => {
        $crate::macros::insert_element(&mut $stack);
        html_impl! { $stack ($($tail)*) }
    };

    // Traditional tag closing
    ($stack:ident (< / $endtag:ident > $($tail:tt)*)) => {
        $stack.pop();
        html_impl! { $stack ($($tail)*) }
    };

    // "End of paring" rule
    ($stack:ident ()) => {
    };
}

#[macro_export]
macro_rules! html {
    ([$id:expr] $($tail:tt)*) => {
        use web_sys::HtmlElement;

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let mut stack = Vec::new();
        stack.push(document.get_element_by_id($id).unwrap().dyn_into::<HtmlElement>().unwrap());

        html_impl! { stack ($($tail)*) }
    };
}

#[macro_export]
macro_rules! toolbar_button {
    ($icon:literal,$tooltip:literal,$value:literal) => {
        ToolbarButton {
            icon: $icon.to_string(),
            tooltip: $tooltip.to_string(),
            value: $value,
        }
    };
}
