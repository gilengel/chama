use crate::map::map::Map;

use super::{undo::Undo, redo::Redo};

pub trait Action : Undo + Redo + Send + Sync{
    fn execute(&mut self, map: &mut Map) {
        self.redo(map);
    }
}

pub struct MultiAction {
    pub actions: Vec<Box<dyn Action>>
}

impl MultiAction {
    pub fn new() -> Self {
        MultiAction {
            actions: Vec::new()
        }
    }
}

impl Action for MultiAction {}


impl Redo for MultiAction {
    fn redo(&mut self, map: &mut Map) {
        for action in self.actions.iter_mut() {
            (*action).redo(map);
        }
    }
}

impl Undo for MultiAction {
    fn undo(&mut self, map: &mut Map) {
        for action in self.actions.iter_mut() {
            (*action).undo(map);
        }
    }
}


