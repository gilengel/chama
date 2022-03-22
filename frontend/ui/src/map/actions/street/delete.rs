use geo::Coordinate;
use rust_editor::{
    actions::{Action, MultiAction, Redo, Undo},
    gizmo::Id
};
use uuid::Uuid;

use crate::{
    map::{
        actions::intersection::{
            delete::DeleteIntersection, remove_connected_street::RemoveConnectedStreet,
            update::UpdateIntersection,
        },
        map::Map,
    },
};

use super::create::CreateStreet;

pub struct DeleteStreet {
    action_stack: MultiAction<Map>,

    street_id: Uuid,
    start: Coordinate<f64>,
    start_id: Option<Uuid>,
    end: Coordinate<f64>,
    end_id: Option<Uuid>,
}

impl DeleteStreet {
    pub fn new(street_id: Uuid, start: Coordinate<f64>, end: Coordinate<f64>) -> Self {
        DeleteStreet {
            action_stack: MultiAction::new(),
            street_id,
            start,
            start_id: None,
            end,
            end_id: None,
        }
    }
}

impl Undo<Map> for DeleteStreet {
    fn undo(&mut self, map: &mut Map) {
        let mut action = CreateStreet::new_with_id(
            self.start_id.unwrap(),
            self.start,
            self.end_id.unwrap(),
            self.end,
            self.street_id,
        );
        action.execute(map);
    }
}

impl Redo<Map> for DeleteStreet {
    fn redo(&mut self, map: &mut Map) {
        self.action_stack.actions.clear();

        let street = map.streets_mut().remove(&self.street_id).unwrap();
        self.start_id = Some(street.start);
        self.end_id = Some(street.end);

        let mut is_start_empty = false;
        let mut start_id = Uuid::default();
        if let Some(start) = map.intersections_mut().get_mut(&street.start) {
            self.action_stack
                .actions
                .push(Box::new(RemoveConnectedStreet::new(
                    start.id(),
                    self.street_id,
                )));

            is_start_empty = start.get_connected_streets().len() == 1;
            start_id = start.id();
        }

        if is_start_empty {
            self.action_stack
                .actions
                .push(Box::new(DeleteIntersection::new(
                    map.intersection(&start_id).unwrap(),
                )));
        } else {
            self.action_stack
                .actions
                .push(Box::new(UpdateIntersection::new(start_id)));
        }

        let mut is_end_empty = false;
        let mut end_id = Uuid::default();
        if let Some(end) = map.intersections_mut().get_mut(&street.end) {
            self.action_stack
                .actions
                .push(Box::new(RemoveConnectedStreet::new(
                    end.id(),
                    self.street_id,
                )));

            is_end_empty = end.get_connected_streets().len() == 1;
            end_id = end.id();
        }

        if is_end_empty {
            self.action_stack
                .actions
                .push(Box::new(DeleteIntersection::new(
                    map.intersection(&end_id).unwrap(),
                )));
        } else {
            self.action_stack
                .actions
                .push(Box::new(UpdateIntersection::new(end_id)));
        }

        self.action_stack.execute(map);
    }
}

impl Action<Map> for DeleteStreet {}

#[cfg(test)]
mod tests {
    use geo::Coordinate;
    use rust_editor::actions::{Action, Redo, Undo};
    use rust_editor::gizmo::Id;
    use uuid::Uuid;

    use crate::map::{
        actions::street::{create::CreateStreet, delete::DeleteStreet},
        intersection::Intersection,
        map::Map,
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

    fn add_street(start_pos: Coordinate<f64>, end_pos: Coordinate<f64>, map: &mut Map) {
        let start_intersection_id = add_intersection(start_pos, map);
        let end_intersection_id = add_intersection(end_pos, map);

        let mut action = CreateStreet::new(
            start_intersection_id,
            start_pos,
            end_intersection_id,
            end_pos,
        );
        action.execute(map);
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

        let start_pos = Coordinate { x: 100., y: 100. };
        let end_pos = Coordinate { x: 300., y: 100. };
        let start_intersection_id = add_intersection(start_pos, &mut map);
        let end_intersection_id = add_intersection(end_pos, &mut map);

        let mut action = CreateStreet::new(
            start_intersection_id,
            start_pos,
            end_intersection_id,
            end_pos,
        );
        action.execute(&mut map);

        assert_eq!(map.streets.len(), 1);

        let old_street = map.streets.iter().next().unwrap().1.clone();

        let mut action =
            DeleteStreet::new(*map.streets.iter().next().unwrap().0, start_pos, end_pos);
        action.redo(&mut map);

        assert!(map.intersection(&start_intersection_id).is_none());
        assert!(map.intersection(&end_intersection_id).is_none());

        assert!(map.streets.is_empty());
        assert_eq!(map.intersections.len(), 0);

        action.undo(&mut map);

        assert_eq!(map.streets.len(), 1);

        let street = map.streets.iter().next().unwrap().1;
        assert_eq!(old_street.start, street.start);
        assert_eq!(old_street.end, street.end);
        assert_eq!(street.start, start_intersection_id);
        assert_eq!(street.end, end_intersection_id);

        assert_eq!(map.intersections.len(), 2);
        assert!(map.intersection(&start_intersection_id).is_some());
        assert!(map.intersection(&end_intersection_id).is_some());
    }

    #[test]
    fn street_delete_undo_works_multiple_times() {
        let mut map = create_map();

        let start = Coordinate { x: 100., y: 100. };
        let end = Coordinate { x: 300., y: 100. };
        add_street(start, end, &mut map);
        assert_eq!(map.streets.len(), 1);

        let street_id = *map.streets.iter().next().unwrap().0;

        let mut action = DeleteStreet::new(street_id, start, end);
        for _ in 0..10 {
            action.redo(&mut map);
            
            assert_eq!(map.streets.len(), 0);

            action.undo(&mut map);

            assert_eq!(map.streets.len(), 1);
            assert!(map.street(&street_id).is_some());
        }
    }
}
