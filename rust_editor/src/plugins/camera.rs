use geo::Coordinate;
use rust_macro::editor_plugin;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::InformationLayer;

use super::plugin::Plugin;

pub trait Renderer {
    fn render(
        &self,
        context: &CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
    ) -> Result<(), JsValue>;
}

#[editor_plugin]
pub struct Camera {
    #[option(label = "Camera Position", description = "Enables / Disables the grid")]
    position: Coordinate<f64>,

    #[option(
        default = true,
        label = "Camera Enabled",
        description = "Enables / Disables the grid"
    )]
    active: bool,
}

impl Camera {
    pub fn x(&self) -> f64 {
        self.position.x
    }

    pub fn set_x(&mut self, x: f64) {
        self.position.x = x;
    }

    pub fn y(&self) -> f64 {
        self.position.y
    }

    pub fn set_y(&mut self, y: f64) {
        self.position.y = y;
    }

    pub fn position(&self) -> Coordinate<f64> {
        self.position
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl<T> Plugin<T> for Camera
where
    T: Renderer + Default + 'static,

{
    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, button: u32, _data: &mut T) {
        self.active = button == 1;
    }

    fn mouse_move(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        mouse_movement: Coordinate<f64>,
        _data: &mut T,
    ) {
        if !self.active {
            return;
        }

        self.position = self.position + mouse_movement;
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, button: u32, _data: &mut T) {
        if self.active && button == 1 {
            self.active = false;
        }
    }
}
