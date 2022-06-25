use std::fmt;

use rust_editor::actions::{Action, Redo, Undo};
use uuid::Uuid;

use crate::map::{intersection::Direction, map::Map};

pub(crate) struct RemoveConnectedStreet {
    street_id: Uuid,
    street_direction: Option<Direction>,
    intersection_id: Uuid,
}

/*
impl RemoveConnectedStreet {
    pub fn new(intersection_id: Uuid, street_id: Uuid) -> Self {
        RemoveConnectedStreet {
            intersection_id,
            street_id,
            street_direction: None,
        }
    }
}
*/

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

        let (direction, _) = intersection
            .remove_connected_street(&self.street_id)
            .unwrap();
        self.street_direction = Some(direction);
    }
}

impl Action<Map> for RemoveConnectedStreet {}

impl fmt::Display for RemoveConnectedStreet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[remove_connected_street] intersection={}, street={}", self.intersection_id, self.street_id)
    }
}

/*
#[cfg(test)]
mod tests {
    use geo::Coordinate;
    use rust_editor::{actions::{Redo, Undo, Action}, gizmo::Id};
    use uuid::Uuid;

    use crate::map::{
        actions::{intersection::remove_connected_street::RemoveConnectedStreet, street::create::CreateStreet},
        intersection::Intersection, map::Map,
    };

    fn create_map() -> Map {
        let mut map = Map::new(100, 100);

        let start = Coordinate { x: 100., y: 100. };
        let intersection_pos = Coordinate { x: 300., y: 100. };
        let end = Coordinate { x: 500., y: 100. };

        add_street(start, intersection_pos, &mut map);
        add_street(intersection_pos, end, &mut map);
        assert_eq!(map.intersections.len(), 3);

        map
    }

    fn find_intersection<'a>(map: &'a mut Map) -> &'a Intersection {
        let (_, intersection) = map
            .intersections
            .iter()
            .find(|(_, intersection)| intersection.get_connected_streets().len() == 2)
            .unwrap();

        intersection
    }


    fn add_street(start_pos: Coordinate<f64>, end_pos: Coordinate<f64>, map: &mut Map) {
        let mut action = CreateStreet::new(start_pos, end_pos, Uuid::new_v4());
        action.execute(map);
    }

    #[test]
    fn remove_connected_street_redo_works() {
        let mut map = create_map();
        let intersection = find_intersection(&mut map);

        let mut action = RemoveConnectedStreet::new(
            intersection.id(),
            map.streets.values().next().unwrap().id(),
        );
        action.redo(&mut map);

        assert!(map
            .intersections
            .iter()
            .all(|(_, x)| x.get_connected_streets().len() == 1));
    }

    #[test]
    fn remove_connected_street_undo_works() {
        let mut map = create_map();
        let intersection = find_intersection(&mut map);

        let mut action = RemoveConnectedStreet::new(
            intersection.id(),
            map.streets.values().next().unwrap().id(),
        );
        action.redo(&mut map);

        assert!(map
            .intersections
            .iter()
            .all(|(_, x)| x.get_connected_streets().len() == 1));

        action.undo(&mut map);

        find_intersection(&mut map);
    }
}
*/