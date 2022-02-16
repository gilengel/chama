use crate::{
    actions::Action,
};

use super::{camera::Renderer, plugin::Plugin};

pub struct Undo<T> {
    pub stack: Vec<Box<dyn Action<T>>>,
}

impl<T> Default for Undo<T> {
    fn default() -> Self {
        Undo { stack: Vec::new() }
    }
}

impl<T> Undo<T>
where
    T: Renderer,
{
    pub fn push(&mut self, action: Box<dyn Action<T>>) {
        self.stack.push(action);
    }
}

impl<T> Plugin<T> for Undo<T>
where
    T: Renderer + Default + 'static,
{
    fn mouse_down(&mut self, _mouse_pos: geo::Coordinate<f64>, _button: u32, _data: &mut T) {}

    fn mouse_move(
        &mut self,
        _mouse_pos: geo::Coordinate<f64>,
        _mouse_movement: geo::Coordinate<f64>,
        _data: &mut T,
    ) {
    }

    fn mouse_up(&mut self, _mouse_pos: geo::Coordinate<f64>, _button: u32, _data: &mut T) {}
}
