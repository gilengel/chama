use geo::Coordinate;
use rust_editor::{
    actions::{Action, MultiAction, Redo, Undo},
    gizmo::{Id, SetId},
};
use uuid::Uuid;

use crate::map::{
    actions::intersection::{create::CreateIntersection, update::UpdateIntersection},
    intersection::Intersection,
    map::Map,
    street::Street,
};

struct CreateSingleStreet {
    id: Uuid,
    start_intersection_id: Uuid,
    end_intersection_id: Uuid,
}

impl CreateSingleStreet {
    pub fn new(id: Uuid, start_intersection_id: Uuid, end_intersection_id: Uuid) -> Self {
        CreateSingleStreet {
            id,
            start_intersection_id,
            end_intersection_id,
        }
    }
}

impl Undo<Map> for CreateSingleStreet {
    fn undo(&mut self, map: &mut Map) {
        let street = map.streets.remove(&self.id).unwrap();
        map.intersection_mut(&street.start)
            .unwrap()
            .remove_connected_street(&street.id());
        map.intersection_mut(&street.end)
            .unwrap()
            .remove_connected_street(&street.id());

        if map
            .intersection(&street.start)
            .unwrap()
            .get_connected_streets()
            .is_empty()
        {
            map.intersections.remove(&self.start_intersection_id);
        }

        if map
            .intersection(&street.end)
            .unwrap()
            .get_connected_streets()
            .is_empty()
        {
            map.intersections.remove(&self.end_intersection_id);
        }
    }
}

impl Redo<Map> for CreateSingleStreet {
    fn redo(&mut self, map: &mut Map) {
        let start = map.intersection(&self.start_intersection_id).unwrap();
        let end = map.intersection(&self.end_intersection_id).unwrap();

        let mut street = Street::default();
        street.set_id(self.id);
        street.set_start(&start);
        street.set_end(&end);

        if let Some(start) = map.intersection_mut(&self.start_intersection_id) {
            start.add_outgoing_street(&street.id());
        }

        if let Some(end) = map.intersection_mut(&self.end_intersection_id) {
            end.add_incoming_street(&street.id());
        }

        map.add_street(street);
    }
}

impl Action<Map> for CreateSingleStreet {}

pub(crate) struct CreateStreet {
    start_intersection_id: Uuid,
    start_pos: Coordinate<f64>,
    end_intersection_id: Uuid,
    end_pos: Coordinate<f64>,
    street_id: Uuid,

    action_stack: MultiAction<Map>,
}

impl CreateStreet {
    pub fn new(
        start_intersection_id: Uuid,
        start_pos: Coordinate<f64>,
        end_intersection_id: Uuid,
        end_pos: Coordinate<f64>,
    ) -> Self {
        CreateStreet::new_with_id(
            start_intersection_id,
            start_pos,
            end_intersection_id,
            end_pos,
            Uuid::new_v4(),
        )
    }

    pub fn new_with_id(
        start_intersection_id: Uuid,
        start_pos: Coordinate<f64>,
        end_intersection_id: Uuid,
        end_pos: Coordinate<f64>,
        street_id: Uuid,
    ) -> Self {
        CreateStreet {
            start_intersection_id,
            start_pos,
            end_intersection_id,
            end_pos,
            street_id,
            action_stack: MultiAction::new(),
        }
    }
}

impl Undo<Map> for CreateStreet {
    fn undo(&mut self, map: &mut Map) {
        self.action_stack.undo(map);
    }
}

impl Redo<Map> for CreateStreet {
    fn redo(&mut self, map: &mut Map) {
        self.action_stack.actions.clear();

        if map.intersection(&self.start_intersection_id).is_none() {
            self.action_stack
                .actions
                .push(Box::new(CreateIntersection::new_with_id(
                    self.start_pos,
                    self.start_intersection_id,
                )));
        }

        if map.intersection(&self.start_intersection_id).is_none() {
            self.action_stack
                .actions
                .push(Box::new(CreateIntersection::new_with_id(
                    self.end_pos,
                    self.end_intersection_id,
                )));
        }

        self.action_stack
            .actions
            .push(Box::new(CreateSingleStreet::new(
                self.street_id,
                self.start_intersection_id,
                self.end_intersection_id,
            )));

        self.action_stack
            .actions
            .push(Box::new(UpdateIntersection::new(
                self.start_intersection_id,
            )));
        self.action_stack
            .actions
            .push(Box::new(UpdateIntersection::new(self.end_intersection_id)));

        self.action_stack.redo(map);
    }
}

impl Action<Map> for CreateStreet {}

#[cfg(test)]
mod tests {
    use geo::Coordinate;
    use rust_editor::{
        actions::{Redo, Undo},
        gizmo::Id,
    };
    use uuid::Uuid;

    use crate::map::{
        actions::street::create::CreateStreet,
        intersection::{Direction, Intersection},
        map::Map,
        street::Street,
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

    fn add_street(start_id: Uuid, end_id: Uuid, map: &mut Map) -> Uuid {
        let mut street = Street::default();
        street.set_start(map.intersection(&start_id).unwrap());
        street.set_end(map.intersection(&end_id).unwrap());

        let id = street.id();

        map.intersection_mut(&start_id)
            .unwrap()
            .add_outgoing_street(&street.id());
        map.intersection_mut(&end_id)
            .unwrap()
            .add_incoming_street(&street.id());
        map.streets.insert(street.id(), street);

        id
    }

    fn straight_street_action(map: &mut Map) -> CreateStreet {
        let start = add_intersection(Coordinate { x: 100., y: 100. }, map);
        let end = add_intersection(Coordinate { x: 300., y: 100. }, map);

        CreateStreet::new(
            start,
            Coordinate { x: 100., y: 100. },
            end,
            Coordinate { x: 300., y: 100. },
        )
    }

    #[test]
    fn isolated_street_redo_works() {
        let mut map = create_map();

        let mut action = straight_street_action(&mut map);
        action.redo(&mut map);

        assert_eq!(map.intersections.len(), 2);
        assert_eq!(map.streets.len(), 1);
    }

    #[test]
    fn connected_street_at_start_redo_works() {
        let mut map = create_map();

        let start = add_intersection(Coordinate { x: 100., y: 100. }, &mut map);
        let middle = add_intersection(Coordinate { x: 300., y: 100. }, &mut map);
        let end = add_intersection(Coordinate { x: 500., y: 100. }, &mut map);

        let street = add_street(start, middle, &mut map);

        let mut action = CreateStreet::new(
            middle,
            Coordinate { x: 300., y: 100. },
            end,
            Coordinate { x: 500., y: 100. },
        );

        action.redo(&mut map);

        assert_eq!(map.streets.len(), 2);

        assert_eq!(
            map.intersection(&middle)
                .unwrap()
                .get_connected_streets()
                .len(),
            2
        );
        assert!(map
            .intersection(&middle)
            .unwrap()
            .get_connected_streets()
            .iter()
            .any(|(direction, id)| *direction == Direction::Out && *id == action.street_id));
    }

    #[test]
    fn connected_street_at_start_undo_works() {
        let mut map = create_map();

        let start = add_intersection(Coordinate { x: 100., y: 100. }, &mut map);
        let middle = add_intersection(Coordinate { x: 300., y: 100. }, &mut map);
        let end = add_intersection(Coordinate { x: 500., y: 100. }, &mut map);

        let street = add_street(start, middle, &mut map);

        let mut action = CreateStreet::new(
            middle,
            Coordinate { x: 300., y: 100. },
            end,
            Coordinate { x: 500., y: 100. },
        );

        action.redo(&mut map);
        action.undo(&mut map);

        assert_eq!(map.streets.len(), 1);

        assert_eq!(
            map.intersection(&middle)
                .unwrap()
                .get_connected_streets()
                .len(),
            1
        );
        assert!(!map
            .intersection(&middle)
            .unwrap()
            .get_connected_streets()
            .iter()
            .any(|(direction, id)| *direction == Direction::Out && *id == action.street_id));
    }

    #[test]
    fn connected_street_at_end_redo_works() {
        let mut map = create_map();

        let start = add_intersection(Coordinate { x: 100., y: 100. }, &mut map);
        let middle = add_intersection(Coordinate { x: 300., y: 100. }, &mut map);
        let end = add_intersection(Coordinate { x: 500., y: 100. }, &mut map);

        let street = add_street(middle, end, &mut map);
        let mut action = CreateStreet::new(
            start,
            Coordinate { x: 100., y: 100. },
            middle,
            Coordinate { x: 300., y: 100. },
        );

        action.redo(&mut map);

        assert_eq!(map.streets.len(), 2);

        assert_eq!(
            map.intersection(&middle)
                .unwrap()
                .get_connected_streets()
                .len(),
            2
        );
        assert!(map
            .intersection(&middle)
            .unwrap()
            .get_connected_streets()
            .iter()
            .any(|(direction, id)| *direction == Direction::In && *id == action.street_id));
    }

    #[test]
    fn connected_street_at_end_undo_works() {
        let mut map = create_map();

        let start = add_intersection(Coordinate { x: 100., y: 100. }, &mut map);
        let middle = add_intersection(Coordinate { x: 300., y: 100. }, &mut map);
        let end = add_intersection(Coordinate { x: 500., y: 100. }, &mut map);

        add_street(middle, end, &mut map);
        let mut action = CreateStreet::new(
            start,
            Coordinate { x: 100., y: 100. },
            middle,
            Coordinate { x: 300., y: 100. },
        );

        action.redo(&mut map);
        action.undo(&mut map);

        assert_eq!(map.streets.len(), 1);

        assert_eq!(
            map.intersection(&middle)
                .unwrap()
                .get_connected_streets()
                .len(),
            1
        );
    }

    #[test]
    fn connected_street_at_middle_redo_works() {
        let mut map = create_map();

        let start = add_intersection(Coordinate { x: 100., y: 100. }, &mut map);
        let middle = add_intersection(Coordinate { x: 300., y: 100. }, &mut map);
        let middle_2 = add_intersection(Coordinate { x: 500., y: 100. }, &mut map);
        let end = add_intersection(Coordinate { x: 700., y: 100. }, &mut map);

        add_street(start, middle, &mut map);
        add_street(middle_2, end, &mut map);
        let mut action = CreateStreet::new(
            middle,
            Coordinate { x: 300., y: 100. },
            middle_2,
            Coordinate { x: 500., y: 100. },
        );

        action.redo(&mut map);

        assert_eq!(map.streets.len(), 3);

        assert_eq!(
            map.intersection(&middle)
                .unwrap()
                .get_connected_streets()
                .len(),
            2
        );

        assert_eq!(
            map.intersection(&middle_2)
                .unwrap()
                .get_connected_streets()
                .len(),
            2
        );

        assert!(map
            .intersection(&middle)
            .unwrap()
            .get_connected_streets()
            .iter()
            .any(|(direction, id)| *direction == Direction::Out && *id == action.street_id));

        assert!(map
            .intersection(&middle_2)
            .unwrap()
            .get_connected_streets()
            .iter()
            .any(|(direction, id)| *direction == Direction::In && *id == action.street_id));
    }

    #[test]
    fn connected_street_at_middle_undo_works() {
        let mut map = create_map();

        let start = add_intersection(Coordinate { x: 100., y: 100. }, &mut map);
        let middle = add_intersection(Coordinate { x: 300., y: 100. }, &mut map);
        let middle_2 = add_intersection(Coordinate { x: 500., y: 100. }, &mut map);
        let end = add_intersection(Coordinate { x: 700., y: 100. }, &mut map);

        add_street(start, middle, &mut map);
        add_street(middle_2, end, &mut map);
        let mut action = CreateStreet::new(
            middle,
            Coordinate { x: 300., y: 100. },
            middle_2,
            Coordinate { x: 500., y: 100. },
        );

        action.redo(&mut map);
        action.undo(&mut map);

        assert_eq!(map.streets.len(), 2);

        assert_eq!(
            map.intersection(&middle)
                .unwrap()
                .get_connected_streets()
                .len(),
            1
        );

        assert_eq!(
            map.intersection(&middle_2)
                .unwrap()
                .get_connected_streets()
                .len(),
            1
        );
    }

    #[test]
    fn isolated_street_undo_works() {
        let mut map = create_map();

        let mut action = straight_street_action(&mut map);
        action.redo(&mut map);
        action.undo(&mut map);

        assert_eq!(map.intersections.len(), 0);
        assert_eq!(map.streets.len(), 0);
    }

    #[test]
    fn isolated_street_undo_multiple_times_works() {
        let mut map = create_map();

        let mut action = straight_street_action(&mut map);

        for _ in 0..10 {
            action.redo(&mut map);
            action.undo(&mut map);

            assert_eq!(map.intersections.len(), 0);
            assert_eq!(map.streets.len(), 0);
        }
    }
}
