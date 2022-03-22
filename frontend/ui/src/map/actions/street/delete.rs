use geo::Coordinate;
use rust_editor::{actions::{MultiAction, Undo, Redo, Action}, gizmo::Id, log};
use uuid::Uuid;

use crate::map::{map::Map, actions::intersection::{remove_connected_street::RemoveConnectedStreet, delete::DeleteIntersection, update::UpdateIntersection}};

pub(crate) struct DeleteStreet {
    action_stack: MultiAction<Map>,

    street_id: Uuid,
    start: Coordinate<f64>,
    end: Coordinate<f64>,
}

impl DeleteStreet {
    pub fn new(street_id: Uuid, start: Coordinate<f64>, end: Coordinate<f64>) -> Self {
        DeleteStreet {
            action_stack: MultiAction::new(),
            street_id,
            start,
            end,
        }
    }
}

impl Undo<Map> for DeleteStreet {
    fn undo(&mut self, map: &mut Map) {
        map.create_street(&self.start, &self.end, 10.0).execute(map);
    }
}

impl Redo<Map> for DeleteStreet {
    fn redo(&mut self, map: &mut Map) {
        if let Some(street) = map.streets_mut().remove(&self.street_id) {
            let mut is_start_empty = false;
            let mut start_id = Uuid::default();
            if let Some(start) = map.intersections_mut().get_mut(&street.start) {
                self.action_stack.actions.push(Box::new(RemoveConnectedStreet::new(start.id(), self.street_id)));

                is_start_empty = start.get_connected_streets().len() == 1;
                start_id = start.id();
            }

            if is_start_empty {
                self.action_stack.actions.push(Box::new(DeleteIntersection::new(map.intersection(&start_id).unwrap())));
            } else {
                self.action_stack.actions.push(Box::new(UpdateIntersection::new(start_id)));
            }

            let mut is_end_empty = false;
            let mut end_id = Uuid::default();
            if let Some(end) = map.intersections_mut().get_mut(&street.end) {
                self.action_stack.actions.push(Box::new(RemoveConnectedStreet::new(end.id(), self.street_id)));

                is_end_empty = end.get_connected_streets().len() == 1;
                end_id = end.id();
            }

            if is_end_empty {
                self.action_stack.actions.push(Box::new(DeleteIntersection::new(map.intersection(&end_id).unwrap())));
            } else {
                self.action_stack.actions.push(Box::new(UpdateIntersection::new(end_id)));
            }

            self.action_stack.execute(map);
        }
    }
}

impl Action<Map> for DeleteStreet {}

#[cfg(test)]
mod tests {
    use geo::Coordinate;
    use rust_editor::{
        actions::{Action, Redo, Undo},
        gizmo::Id,
    };
    use uuid::Uuid;

    use crate::{map::{actions::street::{create::CreateStreet, delete::DeleteStreet}, intersection::Intersection, map::Map}};

    fn create_map() -> Map {
        Map::new(100, 100)
    }

    fn add_street(start: Coordinate<f64>, end: Coordinate<f64>, map: &mut Map) {
        map.create_street(&start, &end, 10.);
    }



    #[test]
    fn street_delete_redo_works() {
        let mut map = create_map();

        let start = Coordinate { x: 100., y: 100. };
        let end = Coordinate { x: 300., y: 100. };
        add_street(start, end, &mut map);
        assert_eq!(map.streets.len(), 1);
    
        let mut action = DeleteStreet::new(*map.streets.iter().next().unwrap().0, start, end);
        action.redo(&mut map);

        assert!(map.streets.is_empty());
        assert_eq!(map.intersections.len(), 0);
    }

    #[test]
    fn street_delete_undo_works() {
        let mut map = create_map();

        let start = Coordinate { x: 100., y: 100. };
        let end = Coordinate { x: 300., y: 100. };
        add_street(start, end, &mut map);
        assert_eq!(map.streets.len(), 1);
    
        let mut action = DeleteStreet::new(*map.streets.iter().next().unwrap().0, start, end);
        action.redo(&mut map);

        assert!(map.streets.is_empty());
        assert_eq!(map.intersections.len(), 0);

        action.undo(&mut map);

        assert_eq!(map.streets.len(), 1);
        assert_eq!(map.intersections.len(), 2);
    }
}
