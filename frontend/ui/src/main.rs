use map::map::Map;

use plugins::create_district::CreateDistrict;
use plugins::delete_district::DeleteDistrict;
use plugins::delete_street::DeleteStreet;
use plugins::load::Load;
use plugins::map_render::MapRender;
use plugins::save::Save;
use rust_editor::plugins::camera::Camera;
use rust_editor::plugins::redo::Redo;
use rust_editor::plugins::undo::Undo;

use plugins::create_freeform_street::CreateFreeformStreet;
use rust_editor::ui::app::x_launch;

mod map;
mod plugins;
mod algorithm;

fn main() {
    let mut editor = x_launch::<Map>();

    editor.add_plugin(Save::default());
    editor.add_plugin(Load::default());
    editor.add_plugin(Camera::default());
    editor.add_plugin(Undo::<Map>::default());
    editor.add_plugin(Redo::<Map>::default());
    editor.add_plugin(CreateFreeformStreet::default());
    editor.add_plugin(DeleteStreet::default());
    editor.add_plugin(CreateDistrict::default());
    editor.add_plugin(DeleteDistrict::default());
    editor.add_plugin(MapRender::default());
}
