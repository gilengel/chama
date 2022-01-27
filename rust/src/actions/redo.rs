use crate::map::map::Map;

pub trait Redo {
    fn redo(&mut self, map: &mut Map);

    fn is_redoable(&self) -> bool {
        true
    }
}
