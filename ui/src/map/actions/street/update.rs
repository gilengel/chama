use core::fmt;

use rust_editor::{
    actions::{Action, Redo, Undo},
    gizmo::Id,
};
use uuid::Uuid;

use crate::map::map::Map;

pub struct UpdateStreet {
    id: Uuid,
    //connected_streets: Option<Vec<(Direction, Uuid)>>
}

impl UpdateStreet {
    pub fn new(id: Uuid) -> Self {
        UpdateStreet {
            id,
            //connected_streets: None
        }
    }
}

impl Undo<Map> for UpdateStreet {
    fn undo(&mut self, _map: &mut Map) {}
}

impl Redo<Map> for UpdateStreet {
    fn redo(&mut self, map: &mut Map) {
        if let Some(street) = map.streets.remove(&self.id) {
            map.streets.insert(street.id(), street);
        }
    }
}

impl Action<Map> for UpdateStreet {}

impl fmt::Display for UpdateStreet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[update_street] street={}", self.id)
    }
}