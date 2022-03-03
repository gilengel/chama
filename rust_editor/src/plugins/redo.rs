use rust_macro::editor_plugin;

use crate::{actions::Action};

use super::{camera::Renderer, plugin::Plugin};

#[editor_plugin(skip)]
pub struct Redo<Data> {
    #[option(skip)]
    pub stack: Vec<Box<dyn Action<Data>>>,
}


impl<T> Redo<T>
where
    T: Renderer,
{
    pub fn push(&mut self, action: Box<dyn Action<T>>) {
        self.stack.push(action);
    }
}

impl<Data> Plugin<Data> for Redo<Data>
where
    Data: Renderer + Default + 'static,

{

    fn mouse_down(&mut self, _mouse_pos: geo::Coordinate<f64>, _button: u32, _data: &mut Data) {}

    fn mouse_move(
        &mut self,
        _mouse_pos: geo::Coordinate<f64>,
        _mouse_movement: geo::Coordinate<f64>,
        _data: &mut Data
    ) {
    }

    fn mouse_up(&mut self, _mouse_pos: geo::Coordinate<f64>, _button: u32, _data: &mut Data) {}    
}
