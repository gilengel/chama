pub trait Redo<T> {
    fn redo(&mut self, map: &mut T);

    fn is_redoable(&self) -> bool {
        true
    }
}

pub trait Undo<T> {
    fn undo(&mut self, map: &mut T);

    fn is_undoable(&self) -> bool {
        true
    }
}


pub trait Action<T> : Undo<T> + Redo<T> + Send + Sync{
    fn execute(&mut self, map: &mut T) {
        self.redo(map);
    }
}

pub struct MultiAction<T> {
    pub actions: Vec<Box<dyn Action<T>>>
}

impl<T> MultiAction<T> {
    pub fn new() -> Self {
        MultiAction {
            actions: Vec::new()
        }
    }
}

impl<T> Action<T> for MultiAction<T> {}


impl<T> Redo<T> for MultiAction<T> {
    fn redo(&mut self, map: &mut T) {
        for action in self.actions.iter_mut() {
            (*action).redo(map);
        }
    }
}

impl<T> Undo<T> for MultiAction<T> {
    fn undo(&mut self, map: &mut T) {
        for action in self.actions.iter_mut() {
            (*action).undo(map);
        }
    }
}


