use geo::Coordinate;
use rust_editor::{actions::{MultiAction, Undo, Redo, Action}, gizmo::Id};
use uuid::Uuid;

use crate::map::map::Map;

pub(crate) struct DeleteStreet {
    action_stack: MultiAction<Map>,

    street_id: Uuid,
    start: Coordinate<f64>,
    end: Coordinate<f64>,
}

impl DeleteStreet {
    pub fn new(street_id: Uuid, start: Coordinate<f64>, end: Coordinate<f64>) -> Self {
        DeleteStreet {
            action_stack: MultiAction::new(),
            street_id,
            start,
            end,
        }
    }
}

impl Undo<Map> for DeleteStreet {
    fn undo(&mut self, map: &mut Map) {
        map.create_street(&self.start, &self.end, 10.0);
    }
}

impl Redo<Map> for DeleteStreet {
    fn redo(&mut self, map: &mut Map) {
        if let Some(street) = map.streets_mut().remove(&self.street_id) {
            let mut is_start_empty = false;
            let mut start_id = Uuid::default();
            if let Some(start) = map.intersections_mut().get_mut(&street.start) {
                start.remove_connected_street(&self.street_id);

                is_start_empty = start.get_connected_streets().is_empty();
                start_id = start.id();
            }

            if is_start_empty {
                map.remove_intersection(&start_id);
            } else {
                map.update_intersection(&start_id);
            }

            let mut is_end_empty = false;
            let mut end_id = Uuid::default();
            if let Some(end) = map.intersections_mut().get_mut(&street.end) {
                end.remove_connected_street(&self.street_id);

                is_end_empty = end.get_connected_streets().is_empty();
                end_id = end.id();
            }

            if is_end_empty {
                map.remove_intersection(&end_id);
            } else {
                map.update_intersection(&end_id);
            }
        }
    }
}

impl Action<Map> for DeleteStreet {}