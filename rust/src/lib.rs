use std::hash::Hash;

use map::map::Map;
use rust_editor::plugins::redo::Redo;
use rust_editor::plugins::toolbar::ToolbarPlugin;
use rust_editor::plugins::undo::Undo;
use rust_editor::ui::app::{x_launch, ModeProps};
use systems::{
    create_freeform_street_system::CreateFreeFormStreetSystem,
    create_street_system::CreateStreetSystem, delete_street_system::DeleteStreetSystem,
    render_map_system::MapRenderSystem,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

mod map;
mod systems;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Modes {
    CreateSimpleStreet,
    CreateFreeformStreet,
    DeleteStreet,
}



#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let mut editor = x_launch::<Modes, Map>();
    editor.add_plugin::<ToolbarPlugin<Map>>();
    editor.add_plugin::<Undo<Map>>();
    editor.add_plugin::<Redo<Map>>();

    editor.add_mode(
        Modes::CreateSimpleStreet,
        vec![
            Box::new(MapRenderSystem::new()),
            Box::new(CreateStreetSystem::new()),
        ],
        Some(ModeProps {
            icon: "add",
            tooltip: "Create Simple Street",
        }),
    );
    editor.add_mode(
        Modes::CreateFreeformStreet,
        vec![
            Box::new(MapRenderSystem::new()),
            Box::new(CreateFreeFormStreetSystem::new()),
        ],
        Some(ModeProps {
            icon: "brush",
            tooltip: "Create Freeform Street",
        }),
    );
    editor.add_mode(
        Modes::DeleteStreet,
        vec![
            Box::new(MapRenderSystem::new()),
            Box::new(DeleteStreetSystem::new()),
        ],
        Some(ModeProps {
            icon: "remove",
            tooltip: "Delete",
        }),
    );

    editor.activate_mode(Modes::CreateFreeformStreet);

    Ok(())
}
