use std::fmt;

use geo::{Coordinate, Line};
use rust_editor::actions::{Action, MultiAction, Redo, Undo};
use uuid::Uuid;

use crate::map::{actions::intersection::create::CreateIntersection, map::Map};


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
        pts.push((redo_intersection(&self.end_pos), self.end_pos));

        for i in 0..pts.len() {
            if let Some(next) = pts.get(i + 1) {
                let start = pts[i];
                let end = *next;

                self.start_intersection_id = Some(start.0);
                self.end_intersection_id = Some(end.0);
            }
        }

        self.action_stack.redo(map);
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
mod tests {}
