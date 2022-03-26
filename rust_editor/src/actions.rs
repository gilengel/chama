use std::fmt;

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


pub trait Action<T>: Undo<T> + Redo<T> + Send + Sync + fmt::Display {
    fn execute(&mut self, map: &mut T) {
        self.redo(map);
    }
}

pub struct MultiAction<T> {
    pub actions: Vec<Box<dyn Action<T>>>,
}

impl<T> fmt::Display for MultiAction<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "===")?;
        self.actions.iter().fold(Ok(()), |result, action| {
            result.and_then(|_| writeln!(f, "\u{251C}  {}", action))
        })?;
        writeln!(f, "===")
    }
}

impl<T> MultiAction<T> {
    pub fn new() -> Self {
        MultiAction {
            actions: Vec::new(),
        }
    }

    pub fn push<A>(&mut self, action: A) where A: Action<T> + 'static {
        self.actions.push(Box::new(action));
    }

    pub fn clear(&mut self) {
        self.actions.clear();
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
        for action in self.actions.iter_mut().rev() {
            (*action).undo(map);
        }
    }
}
