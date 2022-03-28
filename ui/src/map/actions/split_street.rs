use std::fmt;

use geo::{
    line_intersection::{line_intersection, LineIntersection},
    Coordinate, Line,
};
use rust_editor::actions::{Action, MultiAction, Redo, Undo};
use uuid::Uuid;

use crate::map::{map::Map, street::Street};

use super::{
    intersection::{create::CreateIntersection, update::UpdateIntersection},
    street::{create::CreateSingleStreet, delete::SimpleDeleteStreet},
};

pub struct SplitStreet {
    action_stack: MultiAction<Map>,
    street_id: Uuid,
    split_position: Coordinate<f64>,
    intersection_id: Uuid,
}

impl fmt::Display for SplitStreet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[split_street]\n{}", self.action_stack)
    }
}

impl SplitStreet {
    pub fn new(split_position: Coordinate<f64>, intersection_id: Uuid) -> Self {
        SplitStreet {
            split_position,
            intersection_id,
            street_id: Uuid::default(),
            action_stack: MultiAction::new(),
        }
    }

    fn project_point_onto_middle_of_street(
        &mut self,
        point: Coordinate<f64>,
        street: &Street,
    ) -> Coordinate<f64> {
        let start = street.start();
        let end = street.end();

        let perp = street.perp();

        let line1 = Line::new(start, end);
        let line2 = Line::new(point + perp * -1000.0, point + perp * 1000.0);

        let intersection = line_intersection(line1, line2).unwrap();

        if let LineIntersection::SinglePoint {
            intersection,
            is_proper: _,
        } = intersection
        {
            return intersection;
        }

        point
    }
}

impl Undo<Map> for SplitStreet {
    fn undo(&mut self, map: &mut Map) {
        for action in self.action_stack.actions.iter_mut().rev() {
            action.undo(map);
        }
    }
}

impl Redo<Map> for SplitStreet {
    fn redo(&mut self, map: &mut Map) {
        self.action_stack.clear();

        self.street_id = map
            .get_street_at_position(&self.split_position, &vec![])
            .unwrap();
        let splitted_street = map.street(&self.street_id).unwrap();
        let split_position =
            self.project_point_onto_middle_of_street(self.split_position, splitted_street);

        self.action_stack
            .push(SimpleDeleteStreet::new(self.street_id));

        self.action_stack.push(CreateIntersection::new_with_id(
            split_position,
            self.intersection_id,
        ));

        self.action_stack.push(CreateSingleStreet::new(
            Uuid::new_v4(),
            splitted_street.start,
            self.intersection_id,
        ));

        self.action_stack.push(CreateSingleStreet::new(
            Uuid::new_v4(),
            self.intersection_id,
            splitted_street.end,
        ));

        self.action_stack
            .push(UpdateIntersection::new(splitted_street.start));
        self.action_stack
            .push(UpdateIntersection::new(splitted_street.end));
        self.action_stack
            .push(UpdateIntersection::new(self.intersection_id));

        self.action_stack.redo(map);
    }
}

impl Action<Map> for SplitStreet {}

#[cfg(test)]
mod tests {
    use geo::Coordinate;
    use rust_editor::actions::{Action, Redo, Undo};
    use uuid::Uuid;

    use crate::map::actions::split_street::SplitStreet;
    use crate::map::intersection::{Intersection, Side};
    use crate::map::street::Street;
    use crate::map::{actions::street::create::CreateStreet, map::Map};

    fn create_map() -> Map {
        Map::new(100, 100)
    }

    fn add_street(start_pos: Coordinate<f64>, end_pos: Coordinate<f64>, map: &mut Map) {
        let mut action = CreateStreet::new(start_pos, end_pos, Uuid::new_v4());
        action.execute(map);
    }

    #[test]
    fn display() {
        let street_id = Uuid::new_v4();
        let action = SplitStreet::new(
            Coordinate {
                x: 256. + 128.,
                y: 256.,
            },
            street_id,
        );

        let format = format!("{}", action);

        assert_eq!(format, "[split_street]\n")
    }

    #[test]
    fn split_street_redo_works() {
        let mut map = create_map();

        let start = Coordinate { x: 256., y: 256. };
        let end = Coordinate { x: 512., y: 256. };
        add_street(start, end, &mut map);
        assert_eq!(map.streets.len(), 1);

        let street_id = *map.streets.iter().next().unwrap().0;
        let mut action = SplitStreet::new(
            Coordinate {
                x: 256. + 128.,
                y: 256.,
            },
            street_id,
        );
        action.redo(&mut map);

        assert!(map.street(&street_id).is_none());
        assert_eq!(map.streets.len(), 2);

        assert!(map
            .get_intersection_at_position(
                &Coordinate {
                    x: 256. + 128.,
                    y: 256.
                },
                10.,
                &vec![]
            )
            .is_some());
    }

    #[test]
    fn split_street_keeps_connections_redo_works() {
        let mut map = create_map();

        let start = Coordinate { x: 256., y: 256. };
        let middle_1 = Coordinate { x: 512., y: 256. };
        let middle_2 = Coordinate {
            x: 512. + 256.,
            y: 256.,
        };
        let end = Coordinate { x: 1024., y: 256. };

        let left_street_id = Uuid::new_v4();
        let split_street_id = Uuid::new_v4();
        let right_street_id = Uuid::new_v4();

        CreateStreet::new(start, middle_1, left_street_id).redo(&mut map);
        CreateStreet::new(middle_1, middle_2, split_street_id).redo(&mut map);
        CreateStreet::new(middle_2, end, right_street_id).redo(&mut map);

        let mut action = SplitStreet::new(
            Coordinate {
                x: 512. + 128.,
                y: 256.,
            },
            split_street_id,
        );
        action.redo(&mut map);

        assert!(map
            .street(&left_street_id)
            .unwrap()
            .get_next(Side::Left)
            .is_some());
        assert!(map
            .street(&right_street_id)
            .unwrap()
            .get_previous(Side::Left)
            .is_some());
    }

    #[test]
    fn split_street_undo_works() {
        let mut map = create_map();

        let start = Coordinate { x: 256., y: 256. };
        let end = Coordinate { x: 512., y: 256. };
        add_street(start, end, &mut map);
        assert_eq!(map.streets.len(), 1);

        let street_id = *map.streets.iter().next().unwrap().0;
        let mut action = SplitStreet::new(
            Coordinate {
                x: 256. + 128.,
                y: 256.,
            },
            street_id,
        );
        action.redo(&mut map);
        action.undo(&mut map);

        assert!(map.street(&street_id).is_some());
        assert_eq!(map.streets.len(), 1);

        assert!(map
            .get_intersection_at_position(
                &Coordinate {
                    x: 256. + 128.,
                    y: 256.
                },
                10.,
                &vec![]
            )
            .is_none());
    }
}
