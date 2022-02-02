use std::{cell::RefCell, rc::Rc};

use map::map::Map;
use rust_editor::editor::{
    add_mode, add_toolbar, launch, Editor, Toolbar, ToolbarButton, ToolbarPosition,
};
use systems::{
    create_freeform_street_system::CreateFreeFormStreetSystem,
    create_street_system::CreateStreetSystem, delete_street_system::DeleteStreetSystem,
    render_map_system::MapRenderSystem,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

mod map;
mod systems;

#[macro_use]
extern crate rust_macro;

#[derive(Debug, PartialEq, Eq, Hash)]
enum Modes {
    CreateSimpleStreet,
    CreateFreeformStreet,
    DeleteStreet,
}

thread_local! {
    static EDITOR: Rc<RefCell<Editor<Map>>> = Rc::new(RefCell::new(Editor::new(1920, 1080)));
}

macro_rules! add_mode {
    ($editor:ident, $id:expr, $systems:expr) => {
        add_mode($editor.clone(), $id as u8, $systems);
    };
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
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
        add_toolbar(e.clone(), street_toolbar, ToolbarPosition::Left).expect("couldn't add toolbar to editor. Make sure that the editor instance is correctly created.");

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
        add_toolbar(e.clone(), street_toolbar2, ToolbarPosition::Left).expect("couldn't add toolbar to editor. Make sure that the editor instance is correctly created.");

        add_mode!(e, Modes::CreateSimpleStreet, vec![Box::new(CreateStreetSystem::new())]);
        add_mode!(e, Modes::CreateFreeformStreet, vec![Box::new(MapRenderSystem::new()), Box::new(CreateFreeFormStreetSystem::new())]);
        add_mode!(e, Modes::DeleteStreet, vec![Box::new(DeleteStreetSystem::new())]);

        launch(e.clone()).expect("Could not launch the editor. Make sure that an active html document exist");
    });

    Ok(())
}
