use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    editor::Editor,
    toolbar::{Toolbar, ToolbarPosition},
};

use super::{camera::Renderer, plugin::Plugin};

pub struct ToolbarPlugin<T>
where
    T: Renderer + 'static,
{
    pub toolbars: HashMap<ToolbarPosition, Vec<Toolbar<T>>>,
}

impl<T> Default for ToolbarPlugin<T>
where
    T: Renderer + 'static,
{
    fn default() -> Self {
        let mut toolbars = HashMap::new();
        toolbars.insert(ToolbarPosition::Top, Vec::<Toolbar<T>>::new());
        toolbars.insert(ToolbarPosition::Left, Vec::<Toolbar<T>>::new());
        toolbars.insert(ToolbarPosition::Right, Vec::<Toolbar<T>>::new());
        toolbars.insert(ToolbarPosition::Bottom, Vec::<Toolbar<T>>::new());

        ToolbarPlugin {
            toolbars,
        }
    }
}

impl<T> ToolbarPlugin<T> where T: Renderer {}

impl<T> Plugin<T> for ToolbarPlugin<T>
where
    T: Renderer + Default + 'static,
{
    fn execute(&mut self, _editor: &mut Editor<T>) {}

    fn mouse_down(&mut self, _mouse_pos: geo::Coordinate<f64>, _button: u32, _data: &mut T) {}

    fn mouse_move(
        &mut self,
        _mouse_pos: geo::Coordinate<f64>,
        _mouse_movement: geo::Coordinate<f64>,
        _data: &mut T,
    ) {
    }

    fn mouse_up(&mut self, _mouse_pos: geo::Coordinate<f64>, _button: u32, _data: &mut T) {}

    fn toolbars(&self) -> Vec<Toolbar<T>> {
        vec![]
    }

    fn on_startup(&mut self, editor: Rc<RefCell<Editor<T>>>) {

    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
