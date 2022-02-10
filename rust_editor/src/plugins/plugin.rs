use std::any::Any;

use geo::Coordinate;
use rust_internal::PluginOptions;
use web_sys::CanvasRenderingContext2d;

use super::camera::Renderer;

pub trait Plugin<T>
where
    T: Renderer + 'static,
{
    /// Is used to implement behaviour of the state if the user clicked inside the specified
    /// html element by the statemachine.
    ///
    /// * `x` - x coordinate of the cursor where the click occured
    /// * `y` - x coordinate of the cursor where the click occured
    /// * `button` - The number of the pressed button (0=left, 1=middle, 2=right) [See here for more informations](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button)
    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, _button: u32, _data: &mut T) {}

    /// Is used to implement behaviour of the state if the user moved the cursor inside the
    /// specified html element by the statemaschine.
    ///
    /// * `x` - x coordinate of the cursor where the click occured
    /// * `y` - x coordinate of the cursor where the click occured
    fn mouse_move(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        _mouse_movement: Coordinate<f64>,
        _data: &mut T,
    ) {
    }

    fn render(&self, _context: &CanvasRenderingContext2d) {}

    /// Is used to implement behaviour of the state if the user released a pressed mouse button
    /// inside the specified html element by the statemachine.
    ///
    /// * `x` - x coordinate of the cursor where the click occured
    /// * `y` - x coordinate of the cursor where the click occured
    /// * `button` - The number of the pressed button (0=left, 1=middle, 2=right) [See here for more informations](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button)
    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, _button: u32, _data: &mut T) {}

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_component(&self) -> &dyn PluginOptions;
}

pub trait MuuPlugin<T> = Plugin<T> + PluginOptions where T: Renderer + 'static;