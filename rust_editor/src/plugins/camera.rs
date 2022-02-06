use geo::Coordinate;
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


pub struct Camera {
    position: Coordinate<f64>,

    active: bool,
}

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            position: Coordinate { x: 0., y: 0. },
            active: false,
        }
    }
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

impl<T> Plugin<T> for Camera {
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn execute(&mut self, _editor: &mut crate::editor::Editor<T>) where T: Renderer {
        todo!()
    }
}
