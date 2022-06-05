use std::fmt;

use rust_editor::{actions::{Action, Redo, Undo}, interactive_element::{InteractiveElement, InteractiveElementState}};
use uuid::Uuid;

use crate::map::{map::Map, street::Street};

pub struct DeleteStreet {
    street_id: Uuid,
    street: Option<Street>
}

impl DeleteStreet {
    pub fn new(street_id: Uuid) -> Self {
        DeleteStreet {
            street_id,
            street: None
        }
    }
}

impl Undo<Map> for DeleteStreet {
    fn undo(&mut self, map: &mut Map) {        
        map.add_street(self.street.as_ref().unwrap());
    }
}

impl Redo<Map> for DeleteStreet {
    fn redo(&mut self, map: &mut Map) {
        self.street = Some(map.street(&self.street_id).unwrap().clone());
        self.street.as_mut().unwrap().set_state(InteractiveElementState::Normal);
        
        map.remove_street(self.street.as_ref().unwrap());
    }
}

impl Action<Map> for DeleteStreet {}

impl fmt::Display for DeleteStreet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[delete_street] street={}\n\u{251C}",
            self.street_id
        )
    }
}

#[cfg(test)]
mod tests {}
