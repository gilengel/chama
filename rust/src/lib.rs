use std::{cell::RefCell, rc::Rc};

use map::map::Map;
use rust_editor::{
    editor::{add_toolbar, launch, Editor, Toolbar, ToolbarButton, ToolbarPosition, add_mode, request_animation_frame},
    system::System
};
use systems::{
    create_freeform_street_system::CreateFreeFormStreetSystem,
    create_street_system::CreateStreetSystem, delete_street_system::DeleteStreetSystem,
};
use wasm_bindgen::{prelude::{wasm_bindgen, Closure}, JsValue};

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

#[derive(Debug, PartialEq, Eq, Hash)]
enum Modes {
    CreateSimpleStreet,
    CreateFreeformStreet,
    DeleteStreet,
}

impl std::fmt::Display for Modes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

thread_local! {
    static EDITOR: Rc<RefCell<Editor<Map>>> = Rc::new(RefCell::new(Editor::new(1920, 1080)));
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    //let editor = EDITOR.lock().unwrap();

    EDITOR.with(|e| {
        let street_toolbar = Toolbar {
            buttons: vec![
                ToolbarButton {
                    icon: "add",
                    tooltip: "Create Straight Street",
                    value: Modes::CreateSimpleStreet as u8,
                },
                ToolbarButton {
                    icon: "brush",
                    tooltip: "Freestyle Add Street",
                    value: Modes::CreateFreeformStreet as u8,
                },
                ToolbarButton {
                    icon: "delete",
                    tooltip: "Delete Street",
                    value: Modes::DeleteStreet as u8,
                },
            ],
        };
        add_toolbar(e.clone(), street_toolbar, ToolbarPosition::Left);

        let street_toolbar2 = Toolbar {
            buttons: vec![
                ToolbarButton {
                    icon: "add",
                    tooltip: "Create Straight Street",
                    value: Modes::CreateSimpleStreet as u8,
                },
                ToolbarButton {
                    icon: "brush",
                    tooltip: "Freestyle Add Street",
                    value: Modes::CreateFreeformStreet as u8,
                },
                ToolbarButton {
                    icon: "delete",
                    tooltip: "Delete Street",
                    value: Modes::DeleteStreet as u8,
                },
            ],
        };
        add_toolbar(e.clone(), street_toolbar2, ToolbarPosition::Left);

        add_mode(e.clone(), Modes::CreateSimpleStreet as u8, vec![Box::new(CreateStreetSystem::new())]);
        add_mode(e.clone(), Modes::CreateFreeformStreet as u8, vec![Box::new(CreateFreeFormStreetSystem::new())]);
        add_mode(e.clone(), Modes::DeleteStreet as u8, vec![Box::new(DeleteStreetSystem::new())]);


        launch(e.clone());
    });

    Ok(())
}
