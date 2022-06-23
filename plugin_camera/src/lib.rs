use geo::Coordinate;
use rust_macro::editor_plugin;
use rust_editor::input::mouse;
use rust_editor::plugin::Plugin;

#[editor_plugin]
pub struct Camera {
    #[option(
        skip,
        label = "Camera Position",
        description = "Current position of the camera"
    )]
    position: Coordinate<f64>,

    #[option(
        skip,
        default = false,
        label = "Camera Enabled",
        description = "Enables / Disables the camera"
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
    fn mouse_down(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        button: mouse::Button,
        _: &App<Data>,
    ) -> bool {
        self.active = button == mouse::Button::Middle;
        false
    }

    fn mouse_move(
        &mut self,
        _: Coordinate<f64>,
        mouse_movement: Coordinate<f64>,
        _: mouse::Button,
        _: &mut App<Data>,
    ) -> bool {
        if !self.active {
            return false;
        }

        self.position = self.position + mouse_movement;
        false
    }

    fn mouse_up(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        button: mouse::Button,
        _: &mut App<Data>,
    ) -> bool {
        if self.active && button == mouse::Button::Middle {
            self.active = false;
        }

        false
    }
}

/*
#[cfg(test)]
mod tests {

    use geo::Coordinate;

    use crate::{input::mouse, plugins::plugin::Plugin, ui::app::App};

    use super::Camera;

    #[test]
    fn left_mouse_button_activates_camera() {
        let app = App::<bool>::default();
        let mut camera = Camera::default();

        camera.mouse_down(Coordinate { x: 0., y: 0. }, mouse::Button::Left, &app);

        assert!(camera.active);
    }

    #[test]
    fn not_left_mouse_button_does_not_activates_camera() {
        let app = App::<bool>::default();
        let mut camera = Camera::default();

        camera.mouse_down(Coordinate { x: 0., y: 0. }, mouse::Button::Right, &app);

        assert_eq!(camera.active, false);
    }

    #[test]
    fn move_camera_if_button_1_pressed() {
        let mut app = App::<bool>::default();
        let mut camera = Camera::default();

        camera.set_active(true);
        camera.set_x(128.);
        camera.set_y(128.);
        camera.mouse_move(
            Coordinate { x: 0., y: 0. },
            Coordinate { x: 256., y: 256. },
            mouse::Button::Left,
            &mut app,
        );

        assert_eq!(camera.x(), 256. + 128.);
        assert_eq!(camera.y(), 256. + 128.);
    }

    #[test]
    fn not_move_camera_if_not_button_1_pressed() {
        let mut app = App::<bool>::default();
        let mut camera = Camera::default();

        camera.set_active(false);
        camera.mouse_move(
            Coordinate { x: 0., y: 0. },
            Coordinate { x: 256., y: 256. },
            mouse::Button::Left,
            &mut app,
        );

        assert_eq!(camera.position(), Coordinate { x: 0., y: 0. });
    }

    #[test]
    fn left_mouse_button_deactivates_camera() {
        let mut app = App::<bool>::default();
        let mut camera = Camera::default();

        camera.set_active(true);
        camera.mouse_up(Coordinate { x: 0., y: 0. }, mouse::Button::Left, &mut app);
        assert_eq!(camera.active(), false);
    }

    #[test]
    fn not_left_mouse_button_does_not_deactivates_camera() {
        let mut app = App::<bool>::default();
        let mut camera = Camera::default();

        camera.set_active(true);
        camera.mouse_up(Coordinate { x: 0., y: 0. }, mouse::Button::Right, &mut app);
        assert!(camera.active());
    }
}
*/