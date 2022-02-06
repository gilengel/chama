use std::{rc::Rc, cell::RefCell};

use crate::{actions::Action, log, toolbar::ToolbarButton, editor::{Editor, get_plugin}};

use super::{camera::Renderer, plugin::Plugin, redo::Redo};

pub struct Undo<T> {
    pub stack: Vec<Box<dyn Action<T>>>,
}

impl<T> Default for Undo<T> {
    fn default() -> Self {
        Undo { stack: Vec::new() }
    }
}

impl<T> Undo<T> where T: Renderer {
    pub fn push(&mut self, action: Box<dyn Action<T>>) {
        self.stack.push(action);
    }
}

impl<T> Plugin<T> for Undo<T>
where
    T: Renderer + Default + 'static,
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn execute(&mut self, editor: &mut Editor<T>) {
        if let Some(action) = self.stack.last_mut() {                        
            (**action).undo(editor.data_mut());
        }

        if let Some(redo) = editor.get_plugin_mut::<Redo<T>>() {
            redo.stack.push(self.stack.pop().unwrap());
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
            ToolbarButton::new("undo", "Undo Last Action", 0, |e: Rc<RefCell<Editor<T>>>| {
                  e.as_ref().borrow_mut().execute_plugin::<Undo<T>>();
            }),
        ]
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
