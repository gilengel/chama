use map::map::InformationLayer;
use map::map::Map;

use wasm_bindgen::prelude::wasm_bindgen;

use wasm_bindgen::JsValue;

use web_sys::CanvasRenderingContext2d;

mod interactive_element;
mod states;

mod gizmo;
mod grid;
mod renderer;
mod state;
mod store;
mod style;

mod map;

mod editor;

extern crate alloc;

#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    }
}

#[macro_export]
macro_rules! err {
    ( $( $t:tt )* ) => {
        web_sys::console::error_1(&format!( $( $t )* ).into())
    }
}

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

    active: bool,
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

#[wasm_bindgen(start)]
pub fn main() {
}
 