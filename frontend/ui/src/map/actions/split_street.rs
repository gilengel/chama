use geo::{
    line_intersection::{line_intersection, LineIntersection},
    Coordinate, Line,
};
use rust_editor::actions::{Action, MultiAction, Redo, Undo};
use uuid::Uuid;

use crate::map::{map::Map, street::Street};

use super::{
    intersection::create::CreateIntersection,
    street::{
        create::{CreateSingleStreet},
        delete::{SimpleDeleteStreet},
    },
};

pub struct SplitStreet {
    action_stack: MultiAction<Map>,

    split_position: Coordinate<f64>,
    street_id: Uuid,
    intersection_id: Option<Uuid>,
}

impl SplitStreet {
    pub fn new(split_position: Coordinate<f64>, street_id: Uuid) -> Self {
        SplitStreet {
            split_position,
            street_id,
            intersection_id: None,
            action_stack: MultiAction::new(),
        }
    }

    pub fn new_with_id(
        split_position: Coordinate<f64>,
        street_id: Uuid,
        split_intersection_id: Uuid,
    ) -> Self {
        SplitStreet {
            split_position,
            street_id,
            intersection_id: Some(split_intersection_id),
            action_stack: MultiAction::new(),
        }
    }

    pub fn intersection_id(&self) -> Option<Uuid> {
        self.intersection_id
    }

    fn project_point_onto_middle_of_street(
        &self,
        point: Coordinate<f64>,
        street_id: &Uuid,
        map: &Map,
    ) -> Coordinate<f64> {
        let street: &Street = map.street(street_id).unwrap();

        let start = street.start();
        let end = street.end();

        let perp = street.perp();

        let line1 = Line::new(start, end);
        let line2 = Line::new(point + perp * -1000.0, point + perp * 1000.0);

        if let Some(intersection) = line_intersection(line1, line2) {
            match intersection {
                LineIntersection::SinglePoint {
                    intersection,
                    is_proper: _,
                } => {
                    return intersection;
                }
                _ => return point,
            }
        }

        point
    }
}

impl Undo<Map> for SplitStreet {
    fn undo(&mut self, map: &mut Map) {
        self.action_stack.undo(map);
    }
}

impl Redo<Map> for SplitStreet {
    fn redo(&mut self, map: &mut Map) {
        self.intersection_id = Some(self.intersection_id.unwrap_or_else(|| Uuid::new_v4()));

        self.action_stack.actions.clear();

        let split_position =
            self.project_point_onto_middle_of_street(self.split_position, &self.street_id, &map);

        let splitted_street = map.street(&self.street_id).unwrap();

        self.action_stack
            .push(SimpleDeleteStreet::new(self.street_id));

        self.action_stack.push(CreateIntersection::new_with_id(
            split_position,
            self.intersection_id.unwrap(),
        ));

        self.action_stack.push(CreateSingleStreet::new(
            Uuid::new_v4(),
            splitted_street.start,
            self.intersection_id.unwrap(),
        ));

        self.action_stack.push(CreateSingleStreet::new(
            Uuid::new_v4(),
            self.intersection_id.unwrap(),
            splitted_street.end,
        ));

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
    use crate::map::{actions::street::create::CreateStreet, map::Map};

    fn create_map() -> Map {
        Map::new(100, 100)
    }

    fn add_street(start_pos: Coordinate<f64>, end_pos: Coordinate<f64>, map: &mut Map) {
        let mut action = CreateStreet::new(start_pos, end_pos, Uuid::new_v4());
        action.execute(map);
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