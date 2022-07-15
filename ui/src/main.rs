use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use map::map::Map;

use plugin_camera::Camera;
use plugin_ribbon::RibbonPlugin;
use plugin_toolbar::ToolbarPlugin;
use plugin_ui_components::ComponentsPlugin;
use plugin_undo_redo::{Redo, Undo};
use plugins::create_district::CreateDistrict;
use plugins::delete_district::DeleteDistrict;
use plugins::delete_street::DeleteStreet;
use plugins::load::Load;
use plugins::map_render::MapRender;
use plugins::new::New;
use plugins::reference_image::ReferenceImage;
use plugins::save::Save;
use plugins::settings::Settings;
use plugins::sync::Sync;

use plugins::create_freeform_street::CreateFreeformStreet;
use rust_editor::{
    plugin::PluginWithOptions,
    ui::app::{x_launch, PluginId},
};

mod algorithm;
mod map;
mod plugins;

pub type Plugins<Map> =
    BTreeMap<PluginId, Arc<Mutex<Box<dyn PluginWithOptions<Map> + Send >>>>;
//pub type PluginsVec<Map> = HashMap<PluginId, Rc<RefCell<dyn PluginWithOptions<Map>>>>;

/*
#[allow(dead_code)]
#[allow(unused_mut)]
static PLUGINS: Lazy<Mutex<Plugins<Map>>> = Lazy::new(|| {
    let mut m = BTreeMap::new();
    m.insert(ComponentsPlugin::identifier(), Arc::new(Mutex::new(Box::new(ComponentsPlugin::default()))));

    Mutex::new(m)
});
*/
fn main() {
    let mut editor = x_launch::<Map>();

    editor.add_plugin(ComponentsPlugin::default());
    editor.add_plugin(ToolbarPlugin::default());
    editor.add_plugin(New::default());
    editor.add_plugin(Save::default());
    editor.add_plugin(Load::default());
    editor.add_plugin(Settings::default());
    editor.add_plugin(Camera::default());
    editor.add_plugin(Undo::<Map>::default());
    editor.add_plugin(Redo::<Map>::default());
    editor.add_plugin(CreateFreeformStreet::default());
    editor.add_plugin(DeleteStreet::default());
    editor.add_plugin(CreateDistrict::default());
    editor.add_plugin(DeleteDistrict::default());
    editor.add_plugin(MapRender::default());
    editor.add_plugin(ReferenceImage::default());
    editor.add_plugin(RibbonPlugin::default());
    editor.add_plugin(Sync::default());
}
