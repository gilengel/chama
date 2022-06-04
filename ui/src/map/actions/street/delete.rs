use std::fmt;

use rust_editor::actions::{Action, MultiAction, Redo, Undo};
use uuid::Uuid;

use crate::map::map::Map;

pub struct DeleteStreet {
    action_stack: MultiAction<Map>,

    street_id: Uuid,
}

impl DeleteStreet {
    pub fn new(street_id: Uuid) -> Self {
        DeleteStreet {
            action_stack: MultiAction::new(),
            street_id,
        }
    }
}

impl Undo<Map> for DeleteStreet {
    fn undo(&mut self, map: &mut Map) {
        self.action_stack.undo(map);
    }
}

impl Redo<Map> for DeleteStreet {
    fn redo(&mut self, _: &mut Map) {}
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
mod tests {}
