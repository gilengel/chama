use geo::line_intersection::{line_intersection, LineIntersection};
use geo::prelude::{BoundingRect, Contains, EuclideanDistance};
use geo::{Coordinate, Line, LineString, Polygon, Rect};
use rust_editor::actions::{Action, MultiAction, Redo, Undo};
use rust_editor::gizmo::{GetPosition, Id, SetId, SetPosition};
use rust_editor::interactive_element::{InteractiveElement, InteractiveElementState};
use rust_editor::plugins::camera::Renderer;
use rust_editor::InformationLayer;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::cmp::Ordering;
use std::collections::hash_map::Keys;
use std::collections::HashMap;

use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use super::district::District;
use super::intersection::Intersection;
use super::street::Street;

#[derive(Serialize, Deserialize)]
pub struct Map {
    width: u32,
    height: u32,

    streets: HashMap<Uuid, Street>,
    intersections: HashMap<Uuid, Intersection>,
    districts: HashMap<Uuid, District>,

    bounding_box: Rect<f64>,
}

impl Default for Map {
    fn default() -> Map {
        Map {
            width: 2560,
            height: 1440,
            streets: HashMap::new(),
            intersections: HashMap::new(),
            districts: HashMap::new(),

            bounding_box: Rect::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 0., y: 0. }),
        }
    }
}

impl Renderer for Map {
    fn render(
        &self,
        context: &CanvasRenderingContext2d,
        _: &Vec<InformationLayer>,
    ) -> Result<(), JsValue> {
        //context.translate(camera.x as f64, camera.y as f64)?;

        for (_, district) in &self.districts {
            district.render(&context)?;
        }

        for (_, street) in &self.streets {
            street.render(&context)?;
        }

        for (_, intersection) in &self.intersections {
            intersection.render(&context)?;
        }

        //context.set_transform(1., 0., 0., 1., 0., 0.)?;

        Ok(())
    }
}

impl From<&mut Map> for Polygon<f64> {
    fn from(map: &mut Map) -> Polygon<f64> {
        let v: Vec<Coordinate<f64>> = map
            .intersections()
            .into_iter()
            .map(|x| x.1.position())
            .collect();

        Polygon::new(LineString::from(v), vec![])
    }
}

impl geo::algorithm::concave_hull::ConcaveHull for Map {
    type Scalar = f64;

    fn concave_hull(&self, concavity: Self::Scalar) -> geo::Polygon<Self::Scalar> {
        let v: Vec<Coordinate<Self::Scalar>> = self
            .intersections()
            .into_iter()
            .map(|x| x.1.position())
            .collect();

        let polygon: Polygon<Self::Scalar> = Polygon::new(LineString::from(v), vec![]);

        polygon.concave_hull(concavity)
    }
}

struct CreateStreet {
    start_intersection_id: Uuid,
    end_intersection_id: Uuid,
    street_id: Uuid,
}

impl CreateStreet {
    pub fn new(start_intersection_id: Uuid, end_intersection_id: Uuid) -> Self {
        CreateStreet {
            start_intersection_id,
            end_intersection_id,
            street_id: Uuid::new_v4(),
        }
    }
}

impl Undo<Map> for CreateStreet {
    fn undo(&mut self, map: &mut Map) {
        map.remove_street(&self.street_id);
    }
}

impl Redo<Map> for CreateStreet {
    fn redo(&mut self, map: &mut Map) {
        let start = map.intersection(&self.start_intersection_id).unwrap();
        let end = map.intersection(&self.end_intersection_id).unwrap();

        let mut street = Street::default();
        street.set_id(self.street_id);
        street.set_start(&start);
        street.set_end(&end);

        if let Some(start) = map.intersection_mut(&self.start_intersection_id) {
            start.add_outgoing_street(&street.id());
        }

        if let Some(end) = map.intersection_mut(&self.end_intersection_id) {
            end.add_incoming_street(&street.id());
        }

        Some(map.add_street(street));

        map.update_intersection(&self.start_intersection_id);
        map.update_intersection(&self.end_intersection_id);

        map.update_intersection(&self.start_intersection_id);
        map.update_intersection(&self.end_intersection_id);
    }
}

pub struct SplitStreet {
    split_position: Coordinate<f64>,
    street_id: Uuid,
    new_street_id: Uuid,
    pub intersection_id: Option<Uuid>,
}

impl SplitStreet {
    pub fn new(split_position: Coordinate<f64>, street_id: Uuid) -> Self {
        SplitStreet {
            split_position,
            street_id,
            intersection_id: None,
            new_street_id: Uuid::new_v4(),
        }
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
        let start = map.street(&self.new_street_id).unwrap().start;
        map.intersection_mut(&end)
            .unwrap()
            .add_incoming_street(&self.street_id);
        map.intersection_mut(&start)
            .unwrap()
            .remove_connected_street(&self.street_id);
        map.remove_street(&self.new_street_id);

        let end = map.intersection(&end).unwrap().clone();
        map.street_mut(&self.street_id).unwrap().set_end(&end);

        let end = map.street(&self.street_id).unwrap().end;
        let start = map.street(&self.street_id).unwrap().start;
        map.update_intersection(&end);
        map.update_intersection(&start);
    }
}

impl Redo<Map> for SplitStreet {
    fn redo(&mut self, map: &mut Map) {
        let split_position =
            self.project_point_onto_middle_of_street(self.split_position, &self.street_id, &map);

        let mut new_intersection = Intersection::default();
        let new_intersection_id = new_intersection.id();
        new_intersection.set_position(split_position);

        let splitted_street = map.street(&self.street_id).unwrap();

        let old_end = splitted_street.end;

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
    }
}

impl Action<Map> for SplitStreet {}

impl Action<Map> for CreateStreet {}

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        Map {
            width,
            height,
            ..Default::default()
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns `true` if `self` has no streets, no intersections and no districts
    pub fn is_empty(&self) -> bool {
        self.streets.is_empty() && self.intersections.is_empty() && self.districts.is_empty()
    }

    pub fn intersections(&self) -> &HashMap<Uuid, Intersection> {
        &self.intersections
    }

    pub fn intersections_keys<'a>(&'a self) -> Keys<'a, Uuid, Intersection> {
        self.intersections.keys()
    }

    pub fn districts(&self) -> &HashMap<Uuid, District> {
        &self.districts
    }

    pub fn streets(&self) -> &HashMap<Uuid, Street> {
        &self.streets
    }

    pub fn streets_mut(&mut self) -> &mut HashMap<Uuid, Street> {
        &mut self.streets
    }

    pub fn intersections_by_ids<'a>(
        &'a self,
        ids: &'a Vec<Uuid>,
    ) -> impl Iterator<Item = &'a Intersection> {
        self.intersections
            .values()
            .filter(|intersection| ids.contains(&intersection.id()))
    }

    pub fn intersections_within_rectangle<'a>(
        &'a self,
        rect: &'a Rect<f64>,
    ) -> impl Iterator<Item = &'a Intersection> {
        self.intersections
            .values()
            .filter(|intersection| rect.contains(&intersection.position()))
    }

    pub fn intersections_within_rectangle_mut<'a>(
        &'a mut self,
        rect: &'a Rect<f64>,
    ) -> impl Iterator<Item = &'a mut Intersection> {
        self.intersections
            .values_mut()
            .filter(|intersection| rect.contains(&intersection.position()))
    }

    pub fn intersections_with_state<'a>(
        &'a self,
        state: InteractiveElementState,
    ) -> impl Iterator<Item = &'a Intersection> {
        self.intersections
            .values()
            .filter(move |intersection| intersection.state() == state)
    }

    pub fn intersections_with_state_mut<'a>(
        &'a mut self,
        state: InteractiveElementState,
    ) -> impl Iterator<Item = &'a mut Intersection> {
        self.intersections
            .values_mut()
            .filter(move |intersection| intersection.state() == state)
    }

    pub fn add_street(&mut self, street: Street) -> Uuid {
        let id = street.id();
        self.streets.insert(id, street);

        id
    }

    pub fn add_district(&mut self, district: District) -> Uuid {
        let id = district.id();
        self.districts.insert(id, district);

        id
    }

    pub fn add_intersection(&mut self, intersection: Intersection) -> Uuid {
        let id = intersection.id();
        self.intersections.insert(id, intersection);

        self.update_bounding_box();

        id
    }

    pub fn update_street(&mut self, id: &Uuid) {
        let mut street = self.streets.remove(id).unwrap();
        street.update_geometry(&self.intersections, &self.streets);

        self.streets.insert(street.id(), street);
    }

    pub fn update_intersection(&mut self, id: &Uuid) {
        let intersection = self.intersections.get_mut(id).unwrap();
        intersection.reorder(&mut self.streets);

        let streets = intersection.get_connected_streets().clone();
        for (_, id) in streets {
            self.update_street(&id);
        }
    }

    fn _create_street(
        &mut self,
        start_intersection_id: Uuid,
        end_intersection_id: Uuid,
    ) -> CreateStreet {
        let mut action = CreateStreet::new(start_intersection_id, end_intersection_id);

        action.execute(self);
        action
    }

    pub fn create_street(
        &mut self,
        start: &Coordinate<f64>,
        end: &Coordinate<f64>,
        snapping_offset: f64,
    ) -> Box<dyn Action<Map>> {
        let mut action = MultiAction::new();

        let start_id = match self.get_intersection_at_position(&start, snapping_offset, &vec![]) {
            Some(intersection) => intersection,
            None => match self.get_street_at_position(start, &vec![]) {
                Some(street_id) => {
                    let split_action = self.split_street(*start, &street_id);
                    let id = split_action.intersection_id.unwrap();

                    action.actions.push(Box::new(split_action));

                    id
                }
                None => self.add_intersection(Intersection::new(*start))
                
            },
        };

        let end_id = match self.get_intersection_at_position(&end, snapping_offset, &vec![]) {
            Some(intersection) => intersection,
            None => {
                match self.get_street_at_position(end, &vec![]) {
                    Some(street_id) => {
                        let split_action = self.split_street(*end, &street_id);
                        let id = split_action.intersection_id.unwrap();
                        
                        action.actions.push(Box::new(split_action));

                        id                        
                    }
                    None => self.add_intersection(Intersection::new(*end)),
                }
            },
        };

        match self.line_intersection_with_street(&Line::new(*start, *end)) {
            Some((street_id, intersection)) => {
                let split_action = self.split_street(intersection, &street_id);
                let split_id = split_action.intersection_id.unwrap();
                action.actions.push(Box::new(split_action));

                action
                    .actions
                    .push(Box::new(self._create_street(start_id, split_id)));
                action
                    .actions
                    .push(Box::new(self._create_street(split_id, end_id)));
            }
            None => {
                action
                    .actions
                    .push(Box::new(self._create_street(start_id, end_id)));
            }
        };

        Box::new(action)
    }

    pub fn split_street(
        &mut self,
        split_position: Coordinate<f64>,
        street_id: &Uuid,
    ) -> SplitStreet {
        let mut action = SplitStreet::new(split_position, *street_id);
        action.execute(self);

        action
    }

    pub fn get_intersection_at_position(
        &self,
        position: &Coordinate<f64>,
        offset: f64,
        ignored_intersections: &Vec<Uuid>,
    ) -> Option<Uuid> {
        for (id, intersection) in &self.intersections {
            if ignored_intersections.into_iter().any(|e| e == id) {
                continue;
            }

            if intersection.position().euclidean_distance(position) < offset {
                return Some(*id);
            }
        }

        None
    }

    pub fn line_intersection_with_street(
        &self,
        line: &Line<f64>,
    ) -> Option<(Uuid, Coordinate<f64>)> {
        let mut intersections: Vec<(Uuid, Coordinate<f64>)> = Vec::new();

        for (_, street) in &self.streets {
            if let Some(line_intersection) = street.intersect_with_line(line) {
                match line_intersection {
                    LineIntersection::SinglePoint {
                        intersection,
                        is_proper,
                    } => {
                        if is_proper {
                            intersections.push((street.id(), intersection));
                        }
                    }
                    _ => {}
                }
            }
        }

        intersections.sort_by(|a, b| {
            let d1 = a.1.euclidean_distance(&line.start);
            let d2 = b.1.euclidean_distance(&line.start);

            if d1 < d2 {
                return Ordering::Less;
            }

            if d1 == d2 {
                return Ordering::Equal;
            }

            Ordering::Greater
        });

        if intersections.is_empty() {
            return None;
        }

        Some(intersections.first().unwrap().clone())
    }
    pub fn intersection_with_street(&self, street: &Street) -> Option<Coordinate<f64>> {
        let mut intersections = vec![];

        for (_, another_street) in &self.streets {
            if let Some(line_intersection) = street.intersect_with_street(another_street) {
                match line_intersection {
                    LineIntersection::SinglePoint {
                        intersection,
                        is_proper,
                    } => {
                        if is_proper {
                            intersections.push(intersection);
                        }
                    }
                    _ => {}
                }
            }
        }

        intersections.sort_by(|a, b| {
            let d1 = a.euclidean_distance(&street.start());
            let d2 = b.euclidean_distance(&street.start());

            if d1 < d2 {
                return Ordering::Less;
            }

            if d1 == d2 {
                return Ordering::Equal;
            }

            Ordering::Greater
        });

        if intersections.is_empty() {
            return None;
        }

        Some(intersections.first().unwrap().clone())
    }

    pub fn get_intersection_contained_in_polygon(&self, polygon: &Polygon<f64>) -> Option<Uuid> {
        for (id, intersection) in &self.intersections {
            if polygon.contains(&intersection.position()) {
                return Some(*id);
            }
        }

        None
    }

    pub fn get_street_at_position(
        &self,
        position: &Coordinate<f64>,
        ignored_streets: &Vec<Uuid>,
    ) -> Option<Uuid> {
        for (id, street) in &self.streets {
            if ignored_streets.contains(id) {
                continue;
            }

            if street.is_point_on_street(position) {
                return Some(*id);
            }
        }

        None
    }

    pub fn get_nearest_street_to_position(&self, position: &Coordinate<f64>) -> Option<&Street> {
        if let Some((_, nearest_street)) = self.streets.iter().min_by(|(_, x), (_, y)| {
            let x = x.line;
            let y = y.line;

            let d1 = x.euclidean_distance(position);
            let d2 = y.euclidean_distance(position);
            if d1 < d2 {
                return Ordering::Less;
            }

            if d1 > d2 {
                return Ordering::Greater;
            }

            Ordering::Equal
        }) {
            return Some(nearest_street);
        }

        None
    }

    pub fn update_bounding_box(&mut self) {
        let polygon: Polygon<f64> = self.into();
        if let Some(bb) = polygon.bounding_rect() {
            self.bounding_box = bb;

            let offset = 20.0;
            self.bounding_box.set_min(
                self.bounding_box.min()
                    - Coordinate {
                        x: offset,
                        y: offset,
                    },
            );
            self.bounding_box.set_max(
                self.bounding_box.max()
                    + Coordinate {
                        x: offset,
                        y: offset,
                    },
            )
        }
    }

    pub fn street(&self, id: &Uuid) -> Option<&Street> {
        if self.streets.contains_key(id) {
            return Some(self.streets.get(id).unwrap());
        }

        None
    }

    pub fn street_mut(&mut self, id: &Uuid) -> Option<&mut Street> {
        if self.streets.contains_key(id) {
            return Some(self.streets.get_mut(id).unwrap());
        }

        None
    }

    pub fn intersection(&self, id: &Uuid) -> Option<&Intersection> {
        if self.intersections.contains_key(id) {
            return Some(self.intersections.get(id).unwrap());
        }

        None
    }

    pub fn intersection_mut(&mut self, id: &Uuid) -> Option<&mut Intersection> {
        if self.intersections.contains_key(id) {
            return Some(self.intersections.get_mut(id).unwrap());
        }

        None
    }

    pub fn district(&self, id: &Uuid) -> Option<&District> {
        if self.districts.contains_key(id) {
            return Some(self.districts.get(id).unwrap());
        }

        None
    }

    pub fn district_mut(&mut self, id: &Uuid) -> Option<&mut District> {
        if self.districts.contains_key(id) {
            return Some(self.districts.get_mut(id).unwrap());
        }

        None
    }

    pub fn get_district_at_position(&self, position: &Coordinate<f64>) -> Option<Uuid> {
        for (_, district) in &self.districts {
            if district.is_point_on_district(position) {
                return Some(district.id());
            }
        }

        None
    }

    pub fn remove_street(&mut self, id: &Uuid) -> Option<Street> {
        if let Some(street) = self.streets.remove(id) {
            let mut is_start_empty = false;
            let mut start_id = Uuid::default();
            if let Some(start) = self.intersections.get_mut(&street.start) {
                start.remove_connected_street(id);

                is_start_empty = start.get_connected_streets().is_empty();
                start_id = start.id();
            }

            if is_start_empty {
                self.remove_intersection(&start_id);
            } else {
                self.update_intersection(&start_id);
            }

            let mut is_end_empty = false;
            let mut end_id = Uuid::default();
            if let Some(end) = self.intersections.get_mut(&street.end) {
                end.remove_connected_street(id);

                is_end_empty = end.get_connected_streets().is_empty();
                end_id = end.id();
            }

            if is_end_empty {
                self.remove_intersection(&end_id);
            } else {
                self.update_intersection(&end_id);
            }

            return Some(street);
        }

        None
    }

    pub fn remove_intersection(&mut self, id: &Uuid) -> Option<Intersection> {
        if let Some(removed) = self.intersections.remove(id) {
            self.update_bounding_box();

            return Some(removed);
        }

        None
    }

    pub fn remove_district(&mut self, id: &Uuid) {
        self.districts.remove(id);
    }

    pub fn streets_intersecting_ray(
        &self,
        ray_start_pos: &Coordinate<f64>,
        ray_direction: &Coordinate<f64>,
        ray_length: f64,
    ) -> Vec<Uuid> {
        let line = Line::new(*ray_start_pos, *ray_direction * ray_length);
        let mut intersected_streets = vec![];
        for (_, street) in &self.streets {
            let s: &Line<f64> = street.into();
            if let Some(intersection) = line_intersection(*s, line) {
                match intersection {
                    LineIntersection::SinglePoint {
                        intersection: _,
                        is_proper: _,
                    } => {
                        intersected_streets.push(street.id());
                    }
                    _ => {}
                }
            }
        }

        intersected_streets
    }
}
