use rust_macro::editor_plugin;

use crate::actions::Action;

use super::plugin::Plugin;

#[editor_plugin(skip)]
pub struct Undo<Data> {
    #[option(skip)]
    pub stack: Vec<Box<dyn Action<Data>>>,
}

impl<T> Undo<T> {
    pub fn push(&mut self, action: Box<dyn Action<T>>) {
        self.stack.push(action);
    }
}

impl<Data> Plugin<Data> for Undo<Data> where Data: Default + 'static {}
