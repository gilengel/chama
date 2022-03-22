use geo::{Coordinate, line_intersection::{LineIntersection, line_intersection}, Line};
use rust_editor::{actions::{Action, Redo, Undo, MultiAction}};
use uuid::Uuid;

use crate::map::{map::Map, street::Street};

pub struct SplitStreet {
    action_stack: MultiAction<Map>,

    split_position: Coordinate<f64>,
    street_id: Uuid,
    new_street_id: Uuid,
    intersection_id: Option<Uuid>,
}

impl SplitStreet {
    pub fn new(split_position: Coordinate<f64>, street_id: Uuid) -> Self {
        SplitStreet {
            split_position,
            street_id,
            intersection_id: None,
            new_street_id: Uuid::new_v4(),
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
        if map.street(&self.new_street_id).is_none() {
            return;
        }

        let end = map.street(&self.new_street_id).unwrap().end;
        if let Some(end) = map.intersection_mut(&end) {
            end.add_incoming_street(&self.street_id);
        }

        let start = map.street(&self.new_street_id).unwrap().start;
        if let Some(start) = map.intersection_mut(&start) {
            start.remove_connected_street(&self.street_id);
        }

        map.remove_street(&self.new_street_id);

        //let end = map.intersection(&end).unwrap().clone();

        if let Some(end) = map.intersection(&end) {
            let end = end.clone();
            if let Some(street) = map.street_mut(&self.street_id) {
                street.set_end(&end);

                let end = map.street(&self.street_id).unwrap().end;
                let start = map.street(&self.street_id).unwrap().start;
                map.update_intersection(&end);
                map.update_intersection(&start);
            }
        }
    }
}

impl Redo<Map> for SplitStreet {
    fn redo(&mut self, map: &mut Map) {
        /*
        let split_position =
            self.project_point_onto_middle_of_street(self.split_position, &self.street_id, &map);

        let splitted_street = map.street(&self.street_id).unwrap();

        let old_end = splitted_street.end;

        self.action_stack
            .actions
            .push(map.remove_street(&self.street_id));
        self.action_stack.actions.push(map.create_street(
            &split_position,
            &map.intersection(&old_end).unwrap().position(),
            10.0,
        ));

        
        let mut new_intersection = Intersection::default();
        let new_intersection_id = new_intersection.id();
        new_intersection.set_position(split_position);



        map.street_mut(&self.street_id)
            .unwrap()
            .set_end(&new_intersection);

        new_intersection.add_incoming_street(&self.street_id);



        // The street from new intersection to the old end
        let mut new_street = Street::default();

        new_street.set_id(self.new_street_id);
        new_street.set_start(&new_intersection);
        new_street.set_end(&map.intersection(&old_end).unwrap());
        new_intersection.add_outgoing_street(&new_street.id());

        if let Some(old_end) = map.intersection_mut(&old_end) as Option<&mut Intersection> {
            old_end.remove_connected_street(&self.street_id);
            old_end.add_incoming_street(&new_street.id());
        }

        map.add_street(new_street);
        map.add_intersection(new_intersection);

        map.update_intersection(&new_intersection_id);

        // Prevents visual glitches such as that the new street is not visible until the user moves the cursor
        map.update_street(&self.street_id);
        map.update_street(&self.new_street_id);

        map.update_intersection(&old_end);

        self.intersection_id = Some(new_intersection_id);

        */
    }
}

impl Action<Map> for SplitStreet {}