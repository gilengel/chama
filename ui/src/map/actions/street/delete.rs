use std::fmt;

use geo::Coordinate;
use rust_editor::{
    actions::{Action, MultiAction, Redo, Undo},
};
use uuid::Uuid;

use crate::map::{
    actions::intersection::{
        create::CreateIntersection,
        update::UpdateIntersection,
    },
    map::Map,
};






pub struct DeleteStreet {
    action_stack: MultiAction<Map>,

    street_id: Uuid,
    start: Option<Coordinate<f64>>,
    start_id: Option<Uuid>,
    end: Option<Coordinate<f64>>,
    end_id: Option<Uuid>,
}

impl DeleteStreet {
    pub fn new(street_id: Uuid) -> Self {
        DeleteStreet {
            action_stack: MultiAction::new(),
            street_id,
            start: None,
            start_id: None,
            end: None,
            end_id: None,
        }
    }
}

impl Undo<Map> for DeleteStreet {
    fn undo(&mut self, map: &mut Map) {
        self.action_stack.undo(map);
    }
}

impl Redo<Map> for DeleteStreet {
    fn redo(&mut self, map: &mut Map) {
        
    }
}

impl Action<Map> for DeleteStreet {}

impl fmt::Display for DeleteStreet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[delete_street] street={}\n\u{251C}  {}",
            self.street_id, self.action_stack
        )
    }
}

#[cfg(test)]
mod tests {

}
