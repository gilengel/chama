use geo::Coordinate;
use rust_macro::editor_plugin;

use super::plugin::Plugin;

#[editor_plugin]
pub struct Camera {
    #[option(
        skip,
        label = "Camera Position",
        description = "Enables / Disables the grid"
    )]
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

impl<Data> Plugin<Data> for Camera
where
    Data: Default + 'static,
{
    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, button: u32, _data: &mut Data) {
        self.active = button == 1;
    }

    fn mouse_move(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        mouse_movement: Coordinate<f64>,
        _data: &mut Data,
    ) {
        if !self.active {
            return;
        }

        self.position = self.position + mouse_movement;
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, button: u32, _data: &mut Data) {
        if self.active && button == 1 {
            self.active = false;
        }
    }
}
