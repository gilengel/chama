use geo::Coordinate;
use rust_editor::{actions::{Undo, Redo, Action}, gizmo::{Id, GetPosition}};
use uuid::Uuid;

use crate::map::{map::Map, intersection::Intersection};

pub(crate) struct DeleteIntersection {
    id: Uuid,
    position: Coordinate<f64>
}

impl DeleteIntersection {
    pub fn new(intersection: &Intersection) -> Self {
        DeleteIntersection {
            id: intersection.id(),
            position: intersection.position()
        }
    }
}

impl Undo<Map> for DeleteIntersection {
    fn undo(&mut self, map: &mut Map) {
        map.add_intersection(Intersection::new_with_id(self.position, self.id));
    }
}

impl Redo<Map> for DeleteIntersection {
    fn redo(&mut self, map: &mut Map) {
        map.intersections_mut().remove(&self.id);
        /*
        if let Some(removed) =  {
            self.update_bounding_box();

            return Some(removed);
        }

        None
        */

    }
}

impl Action<Map> for DeleteIntersection {}