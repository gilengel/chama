use std::collections::HashMap;

use rust_editor::{
    get_plugin,
    plugins::{
        camera::{Camera, Renderer},
        plugin::PluginWithOptions,
    },
    system::System,
    InformationLayer,
};

use crate::{map::map::Map, Modes};

pub struct MapRenderSystem {}

impl MapRenderSystem {
    pub fn new() -> Self {
        MapRenderSystem {}
    }
}

impl System<Map, Modes> for MapRenderSystem {
    fn render(
        &self,
        map: &Map,
        context: &web_sys::CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
        plugins: &HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) -> Result<(), wasm_bindgen::JsValue> {
        if let Some(camera) = get_plugin::<Map, Modes, Camera>(plugins) {
            context.translate(camera.x() as f64, camera.y() as f64)?;
        }

        map.render(context, additional_information_layer)?;

        context.set_transform(1., 0., 0., 1., 0., 0.)?;

        Ok(())
    }
}
