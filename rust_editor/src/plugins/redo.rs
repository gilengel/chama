use std::{rc::Rc, cell::RefCell};

use crate::{actions::Action, log, toolbar::ToolbarButton, editor::Editor};

use super::{camera::Renderer, plugin::Plugin, undo::Undo};

pub struct Redo<T> {
    pub stack: Vec<Box<dyn Action<T>>>,
}

impl<T> Default for Redo<T> {
    fn default() -> Self {
        Redo { stack: Vec::new() }
    }
}

impl<T> Redo<T> where T: Renderer {
    pub fn push(&mut self, action: Box<dyn Action<T>>) {
        self.stack.push(action);
    }
}

impl<T> Plugin<T> for Redo<T>
where
    T: Renderer + Default + 'static,
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn execute(&mut self, editor: &mut Editor<T>) {
        if let Some(action) = self.stack.last_mut() {                        
            (**action).redo(editor.data_mut());
        }

        if let Some(undo) = editor.get_plugin_mut::<Undo<T>>() {
            undo.stack.push(self.stack.pop().unwrap());
        }
    }

    fn mouse_down(&mut self, _mouse_pos: geo::Coordinate<f64>, _button: u32, _data: &mut T) {}

    fn mouse_move(
        &mut self,
        _mouse_pos: geo::Coordinate<f64>,
        _mouse_movement: geo::Coordinate<f64>,
        _data: &mut T,
    ) {
    }

    fn mouse_up(&mut self, _mouse_pos: geo::Coordinate<f64>, _button: u32, _data: &mut T) {}

    fn toolbar_buttons(&self) -> Vec<ToolbarButton<T>> {
        vec![
            ToolbarButton::new("redo", "Redo Last Action", 1, |e| {
                e.as_ref().borrow_mut().execute_plugin::<Redo<T>>();
            }),
        ]
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
