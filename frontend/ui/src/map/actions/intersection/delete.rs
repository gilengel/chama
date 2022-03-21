use geo::Coordinate;
use rust_editor::{
    actions::{Action, Redo, Undo},
    gizmo::{GetPosition, Id},
};
use uuid::Uuid;

use crate::map::{intersection::Intersection, map::Map};

pub(crate) struct DeleteIntersection {
    id: Uuid,
    position: Coordinate<f64>,
}

impl DeleteIntersection {
    pub fn new(intersection: &Intersection) -> Self {
        DeleteIntersection {
            id: intersection.id(),
            position: intersection.position(),
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

#[cfg(test)]
mod tests {
    use geo::Coordinate;
    use rust_editor::{
        actions::{Action, Redo, Undo},
        gizmo::Id,
    };
    use uuid::Uuid;

    use crate::map::{
        actions::{
            intersection::delete::DeleteIntersection,
            street::{create::CreateStreet, delete::DeleteStreet},
        },
        intersection::Intersection,
        map::Map,
    };

    fn create_map() -> Map {
        Map::new(100, 100)
    }

    fn add_intersection(position: Coordinate<f64>, map: &mut Map) -> Uuid {
        let id = Uuid::new_v4();
        let intersection = Intersection::new_with_id(position, id);

        map.intersections.insert(intersection.id(), intersection);

        id
    }

    #[test]
    fn intersection_delete_redo_works() {
        let mut map = create_map();

        let position = Coordinate { x: 100., y: 100. };
        add_intersection(position, &mut map);
        assert_eq!(map.intersections.len(), 1);

        let mut action = DeleteIntersection::new(map.intersections.values().next().unwrap());
        action.redo(&mut map);

        assert_eq!(map.intersections.len(), 0);
    }

    #[test]
    fn intersection_delete_undo_works() {
        let mut map = create_map();

        let position = Coordinate { x: 100., y: 100. };
        add_intersection(position, &mut map);
        assert_eq!(map.intersections.len(), 1);

        let mut action = DeleteIntersection::new(map.intersections.values().next().unwrap());
        action.redo(&mut map);

        assert_eq!(map.intersections.len(), 0);

        action.undo(&mut map);

        assert_eq!(map.intersections.len(), 1);
    }
}
