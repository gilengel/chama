use rust_editor::actions::{Action, Redo, Undo};
use uuid::Uuid;

use crate::map::{intersection::Direction, map::Map};

pub(crate) struct RemoveConnectedStreet {
    street_id: Uuid,
    street_direction: Option<Direction>,
    intersection_id: Uuid,
}

impl RemoveConnectedStreet {
    pub fn new(intersection_id: Uuid, street_id: Uuid) -> Self {
        RemoveConnectedStreet {
            intersection_id,
            street_id,
            street_direction: None,
        }
    }
}

impl Undo<Map> for RemoveConnectedStreet {
    fn undo(&mut self, map: &mut Map) {
        let intersection = map
            .intersections_mut()
            .get_mut(&self.intersection_id)
            .unwrap();
        
        match self.street_direction.unwrap() {
            Direction::In => intersection.add_incoming_street(&self.street_id),
            Direction::Out => intersection.add_outgoing_street(&self.street_id),
        }
    }
}

impl Redo<Map> for RemoveConnectedStreet {
    fn redo(&mut self, map: &mut Map) {
        let intersection = map
            .intersections_mut()
            .get_mut(&self.intersection_id)
            .unwrap();
        
        let (direction, _) = intersection.remove_connected_street(&self.street_id).unwrap();
        self.street_direction = Some(direction);
    }
}

impl Action<Map> for RemoveConnectedStreet {}
