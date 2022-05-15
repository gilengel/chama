use std::fmt;

use geo::Coordinate;
use rust_editor::{
    actions::{Action, MultiAction, Redo, Undo},
    gizmo::Id, log,
};
use uuid::Uuid;

use crate::map::{
    actions::intersection::{
        create::CreateIntersection, delete::DeleteIntersection,
        update::UpdateIntersection,
    },
    map::Map,
};

use super::create::CreateSingleStreet;

pub struct SimpleDeleteStreet {
    street_id: Uuid,
    start_id: Option<Uuid>,
    start_pos: Option<Coordinate<f64>>,
    end_id: Option<Uuid>,
    end_pos: Option<Coordinate<f64>>,
}

impl SimpleDeleteStreet {
    pub fn new(street_id: Uuid) -> Self {
        SimpleDeleteStreet {
            street_id,
            start_id: None,
            start_pos: None,
            end_id: None,
            end_pos: None,
        }
    }
}

impl Undo<Map> for SimpleDeleteStreet {
    fn undo(&mut self, map: &mut Map) {
        let start_id = self.start_id.unwrap();
        let end_id = self.end_id.unwrap();

        if map.street(&start_id).is_none() {
            CreateIntersection::new_with_id(self.start_pos.unwrap(), start_id).execute(map);
        }

        if map.street(&end_id).is_none() {
            CreateIntersection::new_with_id(self.end_pos.unwrap(), end_id).execute(map);
        }

        CreateSingleStreet::new(self.street_id, start_id, end_id).execute(map);
        UpdateIntersection::new(start_id).execute(map);
        UpdateIntersection::new(end_id).execute(map);
    }
}

impl Redo<Map> for SimpleDeleteStreet {
    fn redo(&mut self, map: &mut Map) {
        let street = map.streets_mut().remove(&self.street_id).unwrap();
        self.start_id = Some(street.start);
        self.start_pos = Some(street.start());
        self.end_id = Some(street.end);
        self.end_pos = Some(street.end());

        map.intersection_mut(&self.start_id.unwrap())
            .unwrap()
            .remove_connected_street(&self.street_id);
        map.intersection_mut(&self.end_id.unwrap())
            .unwrap()
            .remove_connected_street(&self.street_id);
    }
}

impl Action<Map> for SimpleDeleteStreet {}

impl fmt::Display for SimpleDeleteStreet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[delete_simple_street] start={:?} end={:?}", self.start_pos.unwrap().x_y(), self.end_pos.unwrap().x_y())
    }
}

pub struct DeleteStreet {
    action_stack: MultiAction<Map>,

    street_id: Uuid,
    start: Option<Coordinate<f64>>,
    start_id: Option<Uuid>,
    end: Option<Coordinate<f64>>,
    end_id: Option<Uuid>,
}

impl DeleteStreet {
    pub fn new(street_id: Uuid) -> Self {
        DeleteStreet {
            action_stack: MultiAction::new(),
            street_id,
            start: None,
            start_id: None,
            end: None,
            end_id: None,
        }
    }
}

impl Undo<Map> for DeleteStreet {
    fn undo(&mut self, map: &mut Map) {
        self.action_stack.undo(map);
    }
}

impl Redo<Map> for DeleteStreet {
    fn redo(&mut self, map: &mut Map) {
        self.action_stack.actions.clear();


        {
        let street = map.street(&self.street_id).unwrap();
        self.start_id = Some(street.start);
        self.start = Some(street.start());
        self.end_id = Some(street.end);
        self.end = Some(street.end());
        }

        self.action_stack.push(SimpleDeleteStreet::new(self.street_id));

        
        let mut is_start_empty = false;
        let mut start_id = Uuid::default();
        if let Some(start) = map.intersections_mut().get_mut(&self.start_id.unwrap()) {
            is_start_empty = start.get_connected_streets().len() == 1;
            start_id = start.id();
        }

        if is_start_empty {
            self.action_stack.push(DeleteIntersection::new(
                map.intersection(&start_id).unwrap(),
            ));
        } else {
            self.action_stack.push(UpdateIntersection::new(start_id));
        }

        let mut is_end_empty = false;
        let mut end_id = Uuid::default();
        if let Some(end) = map.intersections_mut().get_mut(&self.end_id.unwrap()) {
            is_end_empty = end.get_connected_streets().len() == 1;
            end_id = end.id();
        }

        if is_end_empty {
            self.action_stack
                .push(DeleteIntersection::new(map.intersection(&end_id).unwrap()));
        } else {
            self.action_stack.push(UpdateIntersection::new(end_id));
        }
        

        self.action_stack.execute(map);
    }
}

impl Action<Map> for DeleteStreet {}

impl fmt::Display for DeleteStreet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[delete_street] street={}\n\u{251C}  {}",
            self.street_id, self.action_stack
        )
    }
}

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
        let mut action = CreateStreet::new(start_pos, end_pos, Uuid::new_v4());
        action.execute(map);
    }

    #[test]
    fn street_delete_redo_works() {
        let mut map = create_map();

        let start = Coordinate { x: 100., y: 100. };
        let end = Coordinate { x: 300., y: 100. };
        add_street(start, end, &mut map);
        assert_eq!(map.streets.len(), 1);

        let mut action = DeleteStreet::new(*map.streets.iter().next().unwrap().0);
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

        let mut action = CreateStreet::new(start_pos, end_pos, Uuid::new_v4());
        action.execute(&mut map);

        assert_eq!(map.streets.len(), 1);

        let old_street = map.streets.iter().next().unwrap().1.clone();

        let mut action = DeleteStreet::new(*map.streets.iter().next().unwrap().0);
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

        let mut action = DeleteStreet::new(street_id);
        for _ in 0..10 {
            action.redo(&mut map);

            assert_eq!(map.streets.len(), 0);

            action.undo(&mut map);

            assert_eq!(map.streets.len(), 1);
            assert!(map.street(&street_id).is_some());
        }
    }
}
