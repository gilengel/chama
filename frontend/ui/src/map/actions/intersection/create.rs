use std::fmt;

use geo::Coordinate;
use rust_editor::actions::{Action, Redo, Undo};
use uuid::Uuid;

use crate::map::{intersection::Intersection, map::Map};

pub(crate) struct CreateIntersection {
    position: Coordinate<f64>,
    pub(crate) id: Uuid,
}

impl CreateIntersection {
    pub fn new_with_id(position: Coordinate<f64>, id: Uuid) -> Self {
        CreateIntersection {
            position,
            id
        }
    }
}

impl Undo<Map> for CreateIntersection {
    fn undo(&mut self, map: &mut Map) {
        map.intersections.remove(&self.id);
    }
}

impl Redo<Map> for CreateIntersection {
    fn redo(&mut self, map: &mut Map) {
        map.intersections.insert(self.id, Intersection::new_with_id(self.position, self.id));
    }
}

impl Action<Map> for CreateIntersection {}


impl fmt::Display for CreateIntersection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[create_intersection] position=({},{})", self.position.x, self.position.y)
    }
}