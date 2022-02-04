use rust_editor::{
    editor::get_plugin,
    plugins::camera::{Camera, Renderer},
    system::System,
    InformationLayer,
};
use rust_internal::plugin::Plugin;

use crate::map::map::Map;

pub struct MapRenderSystem {}

impl MapRenderSystem {
    pub fn new() -> Self {
        MapRenderSystem { }
    }
}

impl System<Map> for MapRenderSystem {
    fn render(
        &self,
        map: &Map,
        context: &web_sys::CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
        plugins: &Vec<Box<dyn Plugin<Map>>>,
    ) -> Result<(), wasm_bindgen::JsValue> {
        if let Some(camera) = get_plugin::<Map, Camera>(plugins) {
            context.translate(camera.x() as f64, camera.y() as f64)?;
        }

        map.render(context, additional_information_layer)?;

        context.set_transform(1., 0., 0., 1., 0., 0.)?;

        Ok(())
    }
}
