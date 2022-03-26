use std::fmt;

use rust_editor::{actions::{Undo, Redo, Action}};
use uuid::Uuid;

use crate::map::{map::Map, intersection::Direction, actions::street::update::UpdateStreet};

pub struct UpdateIntersection {
    id: Uuid,
    connected_streets: Option<Vec<(Direction, Uuid)>>
}

impl UpdateIntersection {
    pub fn new(id: Uuid) -> Self {
        UpdateIntersection {
            id,
            connected_streets: None
        }
    }
}

impl Undo<Map> for UpdateIntersection {
    fn undo(&mut self, map: &mut Map) {
        self.redo(map)
    }
}

impl Redo<Map> for UpdateIntersection {
    fn redo(&mut self, map: &mut Map) {
        if let Some(intersection) = map.intersections.get_mut(&self.id) {
            intersection.reorder(&mut map.streets);

            let streets = intersection.get_connected_streets().clone();            
            for (_, id) in &streets {
                UpdateStreet::new(*id).redo(map);
            }

            self.connected_streets = Some(streets);
        }
        
    }
}

impl Action<Map> for UpdateIntersection {}

impl fmt::Display for UpdateIntersection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[update_intersection] id={}", self.id)
    }
}