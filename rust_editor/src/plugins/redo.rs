use rust_macro::editor_plugin;

use crate::{actions::Action};

use super::plugin::Plugin;

#[editor_plugin(skip)]
pub struct Redo<Data> {
    #[option(skip)]
    pub stack: Vec<Box<dyn Action<Data>>>,
}

impl<T> Redo<T> {
    pub fn push(&mut self, action: Box<dyn Action<T>>) {
        self.stack.push(action);
    }
}

impl<Data> Plugin<Data> for Redo<Data>
where
    Data: Default + 'static,
{
}
