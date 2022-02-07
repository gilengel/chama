use std::hash::Hash;
use std::{cell::RefCell, rc::Rc};

use map::map::Map;
use rust_editor::editor::add_plugin;
use rust_editor::launch;
use rust_editor::plugins::redo::Redo;
use rust_editor::plugins::toolbar::ToolbarPlugin;
use rust_editor::plugins::undo::Undo;
use rust_editor::toolbar::{Toolbar, ToolbarPosition, ToolbarRadioButton};
use rust_editor::{
    editor::{add_mode, Editor},
    plugins::camera::Camera,
};
use systems::{
    create_freeform_street_system::CreateFreeFormStreetSystem,
    create_street_system::CreateStreetSystem, delete_street_system::DeleteStreetSystem,
    render_map_system::MapRenderSystem,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

mod map;
mod systems;

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
    ($editor:ident, $id:expr, $systems:expr, $icon:expr, $tooltip:expr) => {
        add_mode($editor.clone(), $id as u8, $systems, $icon, $tooltip);
    };
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    EDITOR.with(|e| {
        add_plugin(e.clone(), ToolbarPlugin::default());
        add_plugin(e.clone(), Camera::default());
        add_plugin(e.clone(), Undo::default());
        add_plugin(e.clone(), Redo::default());

        add_mode!(
            e,
            Modes::CreateSimpleStreet,
            vec![
                Box::new(MapRenderSystem::new()),
                Box::new(CreateStreetSystem::new())
            ],
            "add",
            "Add Straight Street"
        );
        add_mode!(
            e,
            Modes::CreateFreeformStreet,
            vec![
                Box::new(MapRenderSystem::new()),
                Box::new(CreateFreeFormStreetSystem::new())
            ],
            "add",
            "Add Freeform Street"
        );
        add_mode!(
            e,
            Modes::DeleteStreet,
            vec![
                Box::new(MapRenderSystem::new()),
                Box::new(DeleteStreetSystem::new())
            ],
            "delete",
            "Delete Street"
        );

        launch(e.clone())
            .expect("Could not launch the editor. Make sure that an active html document exist");
    });

    Ok(())
}
