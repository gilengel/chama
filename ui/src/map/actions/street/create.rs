use std::fmt;

use geo::{Coordinate, Line};
use rust_editor::{
    actions::{Action, MultiAction, Redo, Undo},
    gizmo::{GetPosition, Id, SetId},
    log,
};
use uuid::Uuid;

use crate::map::{
    actions::{
        intersection::{create::CreateIntersection, update::UpdateIntersection},
        split_street::SplitStreet,
    },
    map::Map,
    street::Street,
};

pub(crate) struct CreateSingleStreet {
    id: Uuid,
    start_id: Uuid,
    end_id: Uuid,

    start_pos: Option<Coordinate<f64>>,
    end_pos: Option<Coordinate<f64>>,
}

impl CreateSingleStreet {
    pub fn new(id: Uuid, start_id: Uuid, end_id: Uuid) -> Self {
        CreateSingleStreet {
            id,
            start_id,
            end_id,
            start_pos: None,
            end_pos: None,
        }
    }
}

impl Undo<Map> for CreateSingleStreet {
    fn undo(&mut self, map: &mut Map) {
        if let Some(street) = map.streets.remove(&self.id) {
            if let Some(start) = map.intersection_mut(&street.start) {
                start.remove_connected_street(&street.id());
            }

            if let Some(end) = map.intersection_mut(&street.end) {
                end.remove_connected_street(&street.id());
            }
        }
    }
}

impl Redo<Map> for CreateSingleStreet {
    fn redo(&mut self, map: &mut Map) {
        let start = map.intersection(&self.start_id).unwrap();
        let end = map.intersection(&self.end_id).unwrap();

        let mut street = Street::default();
        street.set_id(self.id);
        street.set_start(&start);
        street.set_end(&end);

        self.start_pos = Some(start.position());
        self.end_pos = Some(end.position());

        if let Some(start) = map.intersection_mut(&self.start_id) {
            start.add_outgoing_street(&street.id());
        }

        if let Some(end) = map.intersection_mut(&self.end_id) {
            end.add_incoming_street(&street.id());
        }

        street.update_geometry(map.intersections(), map.streets());

        map.add_street(street);
    }
}

impl Action<Map> for CreateSingleStreet {}

impl fmt::Display for CreateSingleStreet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[create_single_street] id={}, start=({},{}), end=({},{})",
            self.id,
            self.start_pos.unwrap().x,
            self.start_pos.unwrap().y,
            self.end_pos.unwrap().x,
            self.end_pos.unwrap().y
        )
    }
}

pub(crate) struct CreateStreet {
    start_intersection_id: Option<Uuid>,
    start_pos: Coordinate<f64>,
    end_intersection_id: Option<Uuid>,
    end_pos: Coordinate<f64>,
    street_ids: Vec<Uuid>,

    action_stack: MultiAction<Map>,
}

impl CreateStreet {
    pub fn new(start_pos: Coordinate<f64>, end_pos: Coordinate<f64>, street_id: Uuid) -> Self {
        CreateStreet {
            start_intersection_id: None,
            start_pos,
            end_intersection_id: None,
            end_pos,
            street_ids: vec![street_id],
            action_stack: MultiAction::new(),
        }
    }
}

impl Undo<Map> for CreateStreet {
    fn undo(&mut self, map: &mut Map) {
        for action in self.action_stack.actions.iter_mut() {
            action.undo(map);
        }
    }
}

impl Redo<Map> for CreateStreet {
    fn redo(&mut self, map: &mut Map) {
        self.action_stack.clear();

        // Check if the new streets intersects an existing intersection, in this case we create two streets one
        // ending in the intersection and the other one starting there
        let new_street_line = Line::new(self.start_pos, self.end_pos);

        // Check if an intersection at start and end exists and if not add CreateIntersection action(s) to the stack
        let mut redo_intersection = |position: &Coordinate<f64>| match map
            .get_intersection_at_position(&position, 1., &vec![])
        {
            Some(intersection) => intersection,
            None => {
                let id = Uuid::new_v4();
                    self.action_stack.push(CreateIntersection::new_with_id(
                        position.clone(),
                        id.clone(),
                    ));

                    id
            }
        };

        let mut pts = vec![];
        pts.push((redo_intersection(&self.start_pos), self.start_pos));

        /*
        let intersections = map.line_intersection_with_intersections(&new_street_line);
        if let Some(intersection) = intersections.first() {
            pts.push(*intersection);
            self.street_ids.push(Uuid::new_v4());
        }
        */

        pts.push((redo_intersection(&self.end_pos), self.end_pos));

        for i in 0..pts.len() {
            if let Some(next) = pts.get(i + 1) {
                let start = pts[i];
                let end = *next;

                self.start_intersection_id = Some(start.0);
                self.end_intersection_id = Some(end.0);

                /*
                let intersections =
                    map.line_intersection_with_streets(&Line::new(start.1, end.1));

                // Split each street were an intersection occurs with the new street in order to properly create districts later on
                if !intersections.is_empty() {
                    let mut intersection_ids: Vec<Uuid> = intersections
                        .iter()
                        .map(|(_, intersection)| {
                            let split_intersection_id = Uuid::new_v4();
                            self.action_stack.push(SplitStreet::new(
                                *intersection,
                                split_intersection_id,
                            ));

                            split_intersection_id
                        })
                        .collect();

                    // Add the original start and end position so that we can create streets for them too
                    intersection_ids.insert(0, start.0);
                    intersection_ids.push(end.0);

                    let mut it = intersection_ids.iter().peekable();
                    while let (Some(current), Some(next)) = (it.next(), it.peek()) {
                        self.action_stack
                          .push(CreateSingleStreet::new(Uuid::new_v4(), *current, **next));
                    }


                    // Update is necessary to recalculate the shape of the newly created streets
                    intersection_ids.iter().for_each(|intersection| {
                        self.action_stack
                            .push(UpdateIntersection::new(*intersection))
                    });


                // Happy path: The new street does not intersects existing streets and we can proceed with creating the street without the need
                // of splitting others
                } else {

                */
                if !map.has_street_connecting_intersections(start.0, end.0) {
                    self.action_stack.push(CreateSingleStreet::new(
                        self.street_ids[i],
                        start.0,
                        end.0,
                    ));
                }

                self.action_stack.push(UpdateIntersection::new(start.0));
                self.action_stack.push(UpdateIntersection::new(end.0));
                //}
            }
        }

        self.action_stack.redo(map);

        //log!("Create street \n{}", self.action_stack);
    }
}

impl Action<Map> for CreateStreet {}

impl fmt::Display for CreateStreet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[create_street] street={:?}\n\u{251C}  {}",
            self.street_ids, self.action_stack
        )
    }
}

#[cfg(test)]
mod tests {
    use geo::Coordinate;
    use rust_editor::{
        actions::{Action, Redo, Undo},
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

    fn straight_street_action() -> CreateStreet {
        CreateStreet::new(
            Coordinate { x: 100., y: 100. },
            Coordinate { x: 300., y: 100. },
            Uuid::new_v4(),
        )
    }

    #[test]
    fn isolated_street_redo_works() {
        let mut map = create_map();

        let mut action = straight_street_action();
        action.redo(&mut map);

        assert_eq!(map.intersections.len(), 2);
        assert_eq!(map.streets.len(), 1);
    }

    #[test]
    fn split_street_at_start_redo_works() {
        let mut map = create_map();

        let old_split_street_id = Uuid::new_v4();
        CreateStreet::new(
            Coordinate { x: 128., y: 128. },
            Coordinate { x: 128., y: 1024. },
            old_split_street_id,
        )
        .execute(&mut map);

        CreateStreet::new(
            Coordinate { x: 128., y: 512. },
            Coordinate { x: 512., y: 512. },
            Uuid::new_v4(),
        )
        .redo(&mut map);

        assert_eq!(map.intersections.len(), 4);
        assert_eq!(map.streets.len(), 3);

        assert_eq!(map.streets.contains_key(&old_split_street_id), false);
    }

    #[test]
    fn split_multiple_intersected_streets_redo_works() {
        let mut map = create_map();

        let mut old_split_street_id = Vec::with_capacity(4);
        for i in 0..4 {
            let id = Uuid::new_v4();
            old_split_street_id.push(id);

            CreateStreet::new(
                Coordinate {
                    x: 128. * (i + 1) as f64,
                    y: 128.,
                },
                Coordinate {
                    x: 128. * (i + 1) as f64,
                    y: 1024.,
                },
                id,
            )
            .execute(&mut map);
        }

        CreateStreet::new(
            Coordinate { x: 0., y: 512. },
            Coordinate { x: 2048., y: 512. },
            Uuid::new_v4(),
        )
        .redo(&mut map);

        assert_eq!(map.intersections.len(), 14);
        assert_eq!(map.streets.len(), 13);
    }

    #[test]
    fn connected_street_at_start_redo_works() {
        let mut map = create_map();

        let middle = add_intersection(Coordinate { x: 300., y: 100. }, &mut map);

        CreateStreet::new(
            Coordinate { x: 100., y: 100. },
            Coordinate { x: 300., y: 100. },
            Uuid::new_v4(),
        )
        .execute(&mut map);

        let mut action = CreateStreet::new(
            Coordinate { x: 300., y: 100. },
            Coordinate { x: 500., y: 100. },
            Uuid::new_v4(),
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
            .any(|(direction, id)| *direction == Direction::Out && *id == action.street_ids[0]));
    }

    #[test]
    fn connected_street_at_start_undo_works() {
        let mut map = create_map();

        let middle = add_intersection(Coordinate { x: 300., y: 100. }, &mut map);

        CreateStreet::new(
            Coordinate { x: 100., y: 100. },
            Coordinate { x: 300., y: 100. },
            Uuid::new_v4(),
        )
        .execute(&mut map);

        let mut action = CreateStreet::new(
            Coordinate { x: 300., y: 100. },
            Coordinate { x: 500., y: 100. },
            Uuid::new_v4(),
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
            .any(|(direction, id)| *direction == Direction::Out && *id == action.street_ids[0]));
    }

    #[test]
    fn connected_street_at_end_redo_works() {
        let mut map = create_map();

        let middle = add_intersection(Coordinate { x: 300., y: 100. }, &mut map);

        CreateStreet::new(
            Coordinate { x: 300., y: 100. },
            Coordinate { x: 500., y: 100. },
            Uuid::new_v4(),
        )
        .execute(&mut map);

        let mut action = CreateStreet::new(
            Coordinate { x: 100., y: 100. },
            Coordinate { x: 300., y: 100. },
            Uuid::new_v4(),
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
            .any(|(direction, id)| *direction == Direction::In && *id == action.street_ids[0]));
    }

    #[test]
    fn connected_street_at_end_undo_works() {
        let mut map = create_map();

        let middle = add_intersection(Coordinate { x: 300., y: 100. }, &mut map);
        let end = add_intersection(Coordinate { x: 500., y: 100. }, &mut map);

        add_street(middle, end, &mut map);
        let mut action = CreateStreet::new(
            Coordinate { x: 100., y: 100. },
            Coordinate { x: 300., y: 100. },
            Uuid::new_v4(),
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
            Coordinate { x: 300., y: 100. },
            Coordinate { x: 500., y: 100. },
            Uuid::new_v4(),
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
            .any(|(direction, id)| *direction == Direction::Out && *id == action.street_ids[0]));

        assert!(map
            .intersection(&middle_2)
            .unwrap()
            .get_connected_streets()
            .iter()
            .any(|(direction, id)| *direction == Direction::In && *id == action.street_ids[0]));
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
            Coordinate { x: 300., y: 100. },
            Coordinate { x: 500., y: 100. },
            Uuid::new_v4(),
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

        let mut action = straight_street_action();
        action.redo(&mut map);
        action.undo(&mut map);

        assert_eq!(map.intersections.len(), 0);
        assert_eq!(map.streets.len(), 0);
    }

    #[test]
    fn isolated_street_undo_multiple_times_works() {
        let mut map = create_map();

        let mut action = straight_street_action();

        for _ in 0..10 {
            action.redo(&mut map);
            action.undo(&mut map);

            assert_eq!(map.intersections.len(), 0);
            assert_eq!(map.streets.len(), 0);
        }
    }
}
