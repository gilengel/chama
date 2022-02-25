use map::map::Map;

use plugins::delete_street::DeleteStreet;
use rust_editor::plugins::camera::Camera;
use rust_editor::plugins::grid::Grid;
use rust_editor::plugins::redo::Redo;
use rust_editor::plugins::undo::Undo;

use plugins::create_freeform_street::CreateFreeformStreet;
use rust_editor::ui::app::x_launch;
use rust_macro::launch;
use wasm_bindgen::prelude::wasm_bindgen;

mod map;
mod plugins;
//mod systems;

#[launch]
fn editor() {
    let mut editor = x_launch::<Map>();

    editor.add_plugin(Camera::default());
    editor.add_plugin(Grid::default());
    editor.add_plugin(Undo::<Map>::default());
    editor.add_plugin(Redo::<Map>::default());
    editor.add_plugin(DeleteStreet::default());
    editor.add_plugin(CreateFreeformStreet::default());
}
