

use std::hash::Hash;

use map::map::Map;

use rust_editor::plugins::camera::Camera;
use rust_editor::plugins::grid::Grid;
use rust_editor::plugins::redo::Redo;
use rust_editor::plugins::undo::Undo;
use rust_editor::ui::app::{x_launch, ModeProps};
use rust_internal::{World};
use systems::create_district_system::CreateDistrictSystem;
use systems::delete_district_system::DeleteDistrictSystem;
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
    CreateDistrict,
    DeleteDistrict    
}


#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let mut editor = x_launch::<Map, Modes>();

    let mut world = World::new();
    let entity = world.new_entity();
    world.add_component_to_entity(entity, Camera::default());

    editor.add_plugin(Camera::default());
    editor.add_plugin(Grid::default());
    editor.add_plugin(Undo::<Map>::default());
    editor.add_plugin(Redo::<Map>::default());

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

    editor.add_mode(
        Modes::CreateDistrict,
        vec![
            Box::new(MapRenderSystem::new()),
            Box::new(CreateDistrictSystem::new()),
        ],
        Some(ModeProps {
            icon: "add",
            tooltip: "Create District",
        }),
    );

    editor.add_mode(
        Modes::DeleteDistrict,
        vec![
            Box::new(MapRenderSystem::new()),
            Box::new(DeleteDistrictSystem::new()),
        ],
        Some(ModeProps {
            icon: "remove",
            tooltip: "Delete District",
        }),
    );

    editor.activate_mode(Modes::CreateFreeformStreet);

    Ok(())
}
