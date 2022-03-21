use rust_editor::{actions::{Action, Redo, Undo}, gizmo::{SetId, Id}};
use uuid::Uuid;

use crate::map::{map::Map, street::Street};

pub(crate) struct CreateStreet {
    start_intersection_id: Uuid,
    end_intersection_id: Uuid,
    street_id: Uuid,
}

impl CreateStreet {
    pub fn new(start_intersection_id: Uuid, end_intersection_id: Uuid) -> Self {
        CreateStreet {
            start_intersection_id,
            end_intersection_id,
            street_id: Uuid::new_v4(),
        }
    }
}

impl Undo<Map> for CreateStreet {
    fn undo(&mut self, map: &mut Map) {
        map.remove_street(&self.street_id).execute(map);
    }
}

impl Redo<Map> for CreateStreet {
    fn redo(&mut self, map: &mut Map) {
        let start = map.intersection(&self.start_intersection_id).unwrap();
        let end = map.intersection(&self.end_intersection_id).unwrap();

        let mut street = Street::default();
        street.set_id(self.street_id);
        street.set_start(&start);
        street.set_end(&end);

        if let Some(start) = map.intersection_mut(&self.start_intersection_id) {
            start.add_outgoing_street(&street.id());
        }

        if let Some(end) = map.intersection_mut(&self.end_intersection_id) {
            end.add_incoming_street(&street.id());
        }

        Some(map.add_street(street));

        map.update_intersection(&self.start_intersection_id);
        map.update_intersection(&self.end_intersection_id);

        map.update_intersection(&self.start_intersection_id);
        map.update_intersection(&self.end_intersection_id);
    }
}

impl Action<Map> for CreateStreet {}
