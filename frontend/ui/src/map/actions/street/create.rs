use rust_editor::{
    actions::{Action, Redo, Undo},
    gizmo::{Id, SetId},
};
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

        map.add_street(street);

        map.update_intersection(&self.start_intersection_id);
        map.update_intersection(&self.end_intersection_id);

        map.update_intersection(&self.start_intersection_id);
        map.update_intersection(&self.end_intersection_id);
    }
}

impl Action<Map> for CreateStreet {}

#[cfg(test)]
mod tests {
    use geo::Coordinate;
    use rust_editor::{
        actions::{Action, Redo, Undo},
        gizmo::Id,
    };
    use uuid::Uuid;

    use crate::map::{actions::street::create::CreateStreet, intersection::Intersection, map::Map};

    fn create_map() -> Map {
        Map::new(100, 100)
    }

    fn add_intersection(position: Coordinate<f64>, map: &mut Map) -> Uuid {
        let id = Uuid::new_v4();
        let intersection = Intersection::new_with_id(position, id);

        map.intersections.insert(intersection.id(), intersection);

        id
    }

    fn straight_street_action(map: &mut Map) -> CreateStreet {
        let start = add_intersection(Coordinate { x: 100., y: 100. }, map);
        let end = add_intersection(Coordinate { x: 300., y: 100. }, map);

        CreateStreet::new(start, end)
    }

    #[test]
    fn street_redo_works() {
        let mut map = create_map();

        let mut action = straight_street_action(&mut map);
        action.redo(&mut map);

        assert_eq!(map.intersections.len(), 2);
        assert_eq!(map.streets.len(), 1);
    }

    #[test]
    fn street_undo_works() {
        let mut map = create_map();

        let mut action = straight_street_action(&mut map);
        action.redo(&mut map);
        action.undo(&mut map);

        assert_eq!(map.intersections.len(), 0);
        assert_eq!(map.streets.len(), 0);
    }
}
