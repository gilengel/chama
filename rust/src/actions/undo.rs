use crate::map::map::Map;

pub trait Undo {
    fn undo(&mut self, map: &mut Map);

    fn is_undoable(&self) -> bool {
        true
    }
}
