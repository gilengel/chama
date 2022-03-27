use std::fmt;

use rust_editor::actions::{Action, Redo, Undo};
use uuid::Uuid;

use crate::map::{actions::street::update::UpdateStreet, intersection::Direction, map::Map};

pub struct UpdateIntersection {
    id: Uuid,
    connected_streets: Option<Vec<(Direction, Uuid)>>,
}

impl UpdateIntersection {
    pub fn new(id: Uuid) -> Self {
        UpdateIntersection {
            id,
            connected_streets: None,
        }
    }
}

impl Undo<Map> for UpdateIntersection {
    fn undo(&mut self, map: &mut Map) {
        self.redo(map)
    }
}

impl Redo<Map> for UpdateIntersection {
    fn redo(&mut self, map: &mut Map) {
        if let Some(intersection) = map.intersections.get_mut(&self.id) {
            intersection.reorder(&mut map.streets);

            let streets = intersection.get_connected_streets().clone();
            for (_, id) in &streets {
                UpdateStreet::new(*id).redo(map);
            }

            self.connected_streets = Some(streets);
        }
    }
}

impl Action<Map> for UpdateIntersection {}

impl fmt::Display for UpdateIntersection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[update_intersection] id={}", self.id)
    }
}

#[cfg(test)]
mod tests {
    use geo::Coordinate;
    use rust_editor::{actions::Action, gizmo::SetId};
    use uuid::Uuid;

    use crate::map::{
        actions::intersection::update::UpdateIntersection,
        intersection::{Intersection, Side},
        map::Map,
        street::Street,
    };

    #[test]
    fn update_intersection_redo_works() {
        let mut map = Map::new(2048, 2048);

        let mut intersection_ids = Vec::<Uuid>::with_capacity(5);
        let mut positions = Vec::<Coordinate<f64>>::with_capacity(5);

        for _ in 0..5 {
            intersection_ids.push(Uuid::new_v4());
        }

        positions.push(Coordinate { x: 0., y: 512. });
        positions.push(Coordinate { x: 512., y: 512. });
        positions.push(Coordinate { x: 512., y: 0. });
        positions.push(Coordinate { x: 1024., y: 512. });
        positions.push(Coordinate { x: 1024., y: 1024. });

        for (id, pos) in intersection_ids.iter().zip(positions.iter()) {
            map.intersections
                .insert(*id, Intersection::new_with_id(*pos, *id));
        }

        let mut street_ids = Vec::<Uuid>::with_capacity(4);

        for _ in 0..4 {
            street_ids.push(Uuid::new_v4());
        }

        // First street: left -> middle
        let mut street = Street::default();
        street.set_id(street_ids[0]);
        street.set_start(map.intersection(&intersection_ids[0]).unwrap());
        street.set_end(map.intersection(&intersection_ids[1]).unwrap());
        map.add_street(street);
        map.intersection_mut(&intersection_ids[1])
            .unwrap()
            .add_incoming_street(&street_ids[0]);

        assert!(map
            .street(&street_ids[0])
            .unwrap()
            .get_next(Side::Left)
            .is_none());
        assert!(map
            .street(&street_ids[0])
            .unwrap()
            .get_next(Side::Right)
            .is_none());

        // Second street: middle -> top
        let mut street = Street::default();
        street.set_id(street_ids[1]);
        street.set_start(map.intersection(&intersection_ids[1]).unwrap());
        street.set_end(map.intersection(&intersection_ids[2]).unwrap());
        map.add_street(street);
        map.intersection_mut(&intersection_ids[1])
            .unwrap()
            .add_outgoing_street(&street_ids[1]);

        assert!(map
            .street(&street_ids[1])
            .unwrap()
            .get_previous(Side::Left)
            .is_none());
        assert!(map
            .street(&street_ids[1])
            .unwrap()
            .get_previous(Side::Right)
            .is_none());

        UpdateIntersection::new(intersection_ids[1]).execute(&mut map);

        assert_eq!(
            map.street(&street_ids[0])
                .unwrap()
                .get_next(Side::Left)
                .unwrap(),
            street_ids[1]
        );
        assert_eq!(
            map.street(&street_ids[0])
                .unwrap()
                .get_next(Side::Right)
                .unwrap(),
            street_ids[1]
        );
        assert_eq!(
            map.street(&street_ids[1])
                .unwrap()
                .get_previous(Side::Left)
                .unwrap(),
            street_ids[0]
        );
        assert_eq!(
            map.street(&street_ids[1])
                .unwrap()
                .get_previous(Side::Right)
                .unwrap(),
            street_ids[0]
        );

        // Third street: right -> middle
        let mut street = Street::default();
        street.set_id(street_ids[2]);
        street.set_start(map.intersection(&intersection_ids[3]).unwrap());
        street.set_end(map.intersection(&intersection_ids[1]).unwrap());
        map.add_street(street);
        map.intersection_mut(&intersection_ids[1])
            .unwrap()
            .add_incoming_street(&street_ids[2]);

        UpdateIntersection::new(intersection_ids[1]).execute(&mut map);

        assert_eq!(
            map.street(&street_ids[0])
                .unwrap()
                .get_next(Side::Left)
                .unwrap(),
            street_ids[1]
        );
        assert_eq!(
            map.street(&street_ids[0])
                .unwrap()
                .get_next(Side::Right)
                .unwrap(),
            street_ids[2]
        );
        assert_eq!(
            map.street(&street_ids[2])
                .unwrap()
                .get_next(Side::Left)
                .unwrap(),
            street_ids[0]
        );
        assert_eq!(
            map.street(&street_ids[2])
                .unwrap()
                .get_next(Side::Right)
                .unwrap(),
            street_ids[1]
        );

        // Fourth street: middle -> bottom
        let mut street = Street::default();
        street.set_id(street_ids[3]);
        street.set_start(map.intersection(&intersection_ids[1]).unwrap());
        street.set_end(map.intersection(&intersection_ids[4]).unwrap());
        map.add_street(street);
        map.intersection_mut(&intersection_ids[1])
            .unwrap()
            .add_outgoing_street(&street_ids[3]);

        UpdateIntersection::new(intersection_ids[1]).execute(&mut map);

        assert_eq!(
            map.street(&street_ids[0])
                .unwrap()
                .get_next(Side::Right)
                .unwrap(),
            street_ids[3]
        );
        assert_eq!(
            map.street(&street_ids[2])
                .unwrap()
                .get_next(Side::Left)
                .unwrap(),
            street_ids[3]
        );
        assert_eq!(
            map.street(&street_ids[3])
                .unwrap()
                .get_previous(Side::Right)
                .unwrap(),
            street_ids[0]
        );
        assert_eq!(
            map.street(&street_ids[3])
                .unwrap()
                .get_previous(Side::Left)
                .unwrap(),
            street_ids[2]
        );
    }
}
