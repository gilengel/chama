use std::{cell::RefCell, rc::Rc, sync::Mutex};

use map::map::Map;
use rust_editor::{
    editor::{Editor, Toolbar, ToolbarButton, ToolbarPosition, add_toolbar},
    toolbar_button, system::System,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use lazy_static::lazy_static;

mod map;
mod systems;

#[macro_use]
extern crate rust_macro;

extern crate alloc;

#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    }
}

#[macro_export]
macro_rules! err {
    ( $( $t:tt )* ) => {
        web_sys::console::error_1(&format!( $( $t )* ).into())
    }
}

type EditorMode = Vec<Box<dyn System<Map> + Send + Sync>>;

enum Modes {
    CreateSimpleStreet(EditorMode),
    CreateFreeformStreet(EditorMode),
    DeleteStreem(EditorMode)
}


lazy_static! {
    static ref EDITOR: Mutex<Editor<Map>> = Mutex::new(Editor::new(1920, 1080));
}

fn add_modes() -> Result<(), JsValue> {
    Ok(())   
}

fn add_toolbars() -> Result<(), JsValue> {
    let street_toolbar = Toolbar {
        buttons: vec![
            toolbar_button! { "add", "Create Straight Street", "0"},
            toolbar_button! { "brush", "Freestyle Add Street", "1"},
            toolbar_button! { "delete", "Delete Street", "2"},
        ],
    };
    add_toolbar(EDITOR.lock().unwrap(), street_toolbar, ToolbarPosition::Left)?;

    let district_toolbar = Toolbar {
        buttons: vec![
            toolbar_button! { "add", "Create District", "0"},
            toolbar_button! { "delete", "Delete District", "1"},
        ],
    };
    add_toolbar(EDITOR.lock().unwrap(), district_toolbar, ToolbarPosition::Left)?; 
    
    Ok(())
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    add_modes()?;
    add_toolbars()?;



    Ok(())
}
