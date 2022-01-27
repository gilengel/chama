use crate::{
    map::map::{InformationLayer, Map},
    state::System,
    Camera, Renderer,
};

pub struct MapRenderSystem {}

impl MapRenderSystem {
    pub fn new() -> Self {
        MapRenderSystem {}
    }
}

impl System for MapRenderSystem {
    fn render(
        &self,
        map: &Map,
        context: &web_sys::CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
        camera: &Camera,
    ) -> Result<(), wasm_bindgen::JsValue> {
        map.render(context, additional_information_layer, camera)?;

        Ok(())
    }
}
