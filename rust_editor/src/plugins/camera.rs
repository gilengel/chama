use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::InformationLayer;

pub trait Renderer {
    fn render(
        &self,
        context: &CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
        camera: &Camera,
    ) -> Result<(), JsValue>;
}

pub struct Camera {
    pub x: i32,
    pub y: i32,

    pub active: bool,
}

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            x: 0,
            y: 0,
            active: false,
        }
    }
}
