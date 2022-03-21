use rust_editor::{actions::{Undo, Redo, Action}};
use uuid::Uuid;

use crate::map::{map::Map, intersection::Direction};

pub(crate) struct UpdateIntersection {
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
    }
}

impl Redo<Map> for UpdateIntersection {
    fn redo(&mut self, map: &mut Map) {
        if let Some(intersection) = map.intersections.get_mut(&self.id) {
            intersection.reorder(&mut map.streets);

            let streets = intersection.get_connected_streets().clone();            
            for (_, id) in &streets {
                map.update_street(id);
            }

            self.connected_streets = Some(streets);
        }
        
    }
}

impl Action<Map> for UpdateIntersection {}