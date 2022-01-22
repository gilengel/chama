use geo::line_intersection::{line_intersection, LineIntersection};
use geo::prelude::{BoundingRect, Contains, EuclideanDistance};
use geo::{Coordinate, Line, LineString, Point, Polygon, Rect};
use serde::{Deserialize, Serialize};

use std::cmp::Ordering;
use std::collections::HashMap;

use uuid::Uuid;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::CanvasRenderingContext2d;

use crate::district::District;
use crate::intersection::{Intersection, Side};
use crate::street::Street;
use crate::{Camera, Renderer, log};

#[derive(Serialize, Deserialize)]
pub struct Map {
    width: u32,
    height: u32,

    streets: HashMap<Uuid, Street>,
    intersections: HashMap<Uuid, Intersection>,
    districts: HashMap<Uuid, District>,

    bounding_box: Rect<f64>,
}

#[derive(PartialEq)]
pub enum InformationLayer {
    Debug,
}

impl Default for Map {
    fn default() -> Map {
        Map {
            width: 1920,
            height: 800,
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
        additional_information_layer: &Vec<InformationLayer>,
        camera: &Camera,
    ) -> Result<(), JsValue> {
        context.translate(camera.x as f64, camera.y as f64)?;

        for (_, district) in &self.districts {
            district.render(&context, additional_information_layer)?;
        }

        for (_, street) in &self.streets {
            street.render(&context, additional_information_layer)?;
        }

        for (_, intersection) in &self.intersections {
            intersection.render(&context, additional_information_layer)?;
        }

        context.set_transform(1., 0., 0., 1., 0., 0.)?;

        Ok(())
    }
}

impl From<&mut Map> for Polygon<f64> {
    fn from(map: &mut Map) -> Polygon<f64> {
        let v: Vec<Coordinate<f64>> = map
            .intersections()
            .into_iter()
            .map(|x| x.1.get_position())
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
            .map(|x| x.1.get_position())
            .collect();

        let polygon: Polygon<Self::Scalar> = Polygon::new(LineString::from(v), vec![]);

        polygon.concave_hull(concavity)
    }
}

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

    pub fn streets(&self) -> &HashMap<Uuid, Street> {
        &self.streets
    }

    pub fn add_street(&mut self, street: Street) -> Uuid {
        let id = street.id;
        self.streets.insert(id, street);

        id
    }

    pub fn add_district(&mut self, district: District) -> Uuid {
        let id = district.id;
        self.districts.insert(id, district);

        id
    }

    pub fn add_intersection(&mut self, intersection: Intersection) -> Uuid {
        let id = intersection.id;
        self.intersections.insert(id, intersection);

        self.update_bounding_box();

        id
    }

    pub fn update_street(&mut self, id: &Uuid) {
        let mut street = self.streets.remove(id).unwrap();
        street.update_geometry(&self.intersections, &self.streets);

        self.streets.insert(street.id, street);
    }

    pub fn update_intersection(&mut self, id: &Uuid) {
        let intersection = self.intersections.get_mut(id).unwrap();
        intersection.reorder(&mut self.streets);

        let streets = intersection.get_connected_streets().clone();
        for (_, id) in streets {
            self.update_street(&id);
        }
    }

    fn _create_street(&mut self, start_id: Uuid, end_id: Uuid) {
        let start = self.intersection(&start_id).unwrap();
        let end = self.intersection(&end_id).unwrap();

        let mut street = Street::default();

        street.set_start(&start);
        street.set_end(&end);

        if let Some(start) = self.intersection_mut(&start_id) {
            start.add_outgoing_street(&street.id);
        }

        if let Some(end) = self.intersection_mut(&end_id) {
            end.add_incoming_street(&street.id);
        }

        self.add_street(street);

        self.update_intersection(&start_id);
        self.update_intersection(&end_id);

        self.update_intersection(&start_id);
        self.update_intersection(&end_id);
    }

    fn project_point_onto_middle_of_street(
        &self,
        point: Coordinate<f64>,
        street_id: &Uuid,
    ) -> Coordinate<f64> {
        let street: &Street = self.street(street_id).unwrap();

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

    pub fn create_street(&mut self, start: &Coordinate<f64>, end: &Coordinate<f64>, snapping_offset: f64) {
        let temp_polygon = |start: Coordinate<f64>, end: Coordinate<f64>| -> Polygon<f64> {
            let length = start.euclidean_distance(&end);
            let vec = end - start;
            let norm = Coordinate {
                x: vec.x / length,
                y: vec.y / length,
            };

            let perp = Point::new(-norm.y, norm.x);
            let offset: Coordinate<f64> = (perp * 10.0).into();

            Polygon::new(
                LineString::from(vec![
                    start,
                    start - offset,
                    end - offset,
                    end,
                    end + offset,
                    start + offset,
                ]),
                vec![],
            )
        };

        let start_id = match self.get_intersection_at_position(&start, snapping_offset, &vec![]) {
            Some(intersection) => intersection,
            None => match self.get_street_at_position(start, &vec![]) {
                Some(street_id) => self.split_street(*start, &street_id),
                None => self.add_intersection(Intersection::new(*start)),
            },
        };

        let end_id = match self.get_intersection_at_position(&end, snapping_offset, &vec![]) {
            Some(intersection) => intersection,
            None => match self.get_street_at_position(end, &vec![]) {
                Some(street_id) => self.split_street(*end, &street_id),
                None => self.add_intersection(Intersection::new(*end)),
            },
        };

        match self.line_intersection_with_street(&Line::new(*start, *end)) {
            Some((street_id, intersection)) => {
                let intersection =
                    self.project_point_onto_middle_of_street(intersection, &street_id);

                let split_id = self.split_street(intersection, &street_id);

                self._create_street(start_id, split_id);
                self._create_street(split_id, end_id);
            }
            None => match self.get_intersection_contained_in_polygon(&temp_polygon(
                self.intersection(&start_id).unwrap().get_position(),
                self.intersection(&end_id).unwrap().get_position(),
            )) {
                Some(split_id) => {
                    self._create_street(start_id, split_id);
                    self._create_street(split_id, end_id);
                }
                None => self._create_street(start_id, end_id),
            },
        };
    }

    pub fn split_street(&mut self, split_position: Coordinate<f64>, street_id: &Uuid) -> Uuid {
        let mut street_id = *street_id;

        let mut new_intersection = Intersection::default();
        let new_intersection_id = new_intersection.id;
        new_intersection.set_position(split_position);

        let splitted_street = self.street(&street_id).unwrap();
        let splitted_street_start_id = splitted_street.start;

        let mut old_end = splitted_street.end;
        let old_start = splitted_street.start;

        if splitted_street.start().euclidean_distance(&split_position) < 40.0 {         

            let conntected_streets = self.intersection(&splitted_street_start_id).unwrap().get_connected_streets().len();
            if conntected_streets  == 2 {
                let previous_street = splitted_street.get_previous(Side::Left).unwrap();

                if let Some(old_start) = self.intersection_mut(&old_start) {
                    old_start.remove_connected_street(&previous_street);
                }
                
                if let Some(end) = self.intersection_mut(&old_end) {
                    end.add_incoming_street(&previous_street);
                }

                let foo = self.intersection(&old_end).unwrap().clone();

                self.street_mut(&previous_street).unwrap().set_end(&foo);
                self.remove_street(&street_id);           
                
                street_id = previous_street.clone();
            }
        }
        
        let splitted_street = self.street(&street_id).unwrap();
        let splitted_street_end_id = splitted_street.end;

        
        if splitted_street.end().euclidean_distance(&split_position) < 80.0 {         
                        
            let conntected_streets = self.intersection(&splitted_street_end_id).unwrap().get_connected_streets().len();

            if conntected_streets  == 2 {
                
                let next_street = splitted_street.get_next(Side::Left).unwrap();
                let next_street_end = self.street(&next_street).unwrap().end;

                if let Some(old_end) = self.intersection_mut(&old_end) {
                    old_end.remove_connected_street(&street_id);
                    
                }
                
                if let Some(next_street_end) = self.intersection_mut(&next_street_end) {
                    next_street_end.add_incoming_street(&street_id);
                }

                let foo = self.intersection(&next_street_end).unwrap().clone();
                self.street_mut(&street_id).unwrap().set_end(&foo);

                log!("{}", foo.id);

                old_end = next_street_end;
                

                self.remove_street(&next_street);      
            }
            
        }

        self.street_mut(&street_id).unwrap().set_end(&new_intersection);


        new_intersection.add_incoming_street(&street_id);

        // The street from new intersection to the old end
        let mut new_street = Street::default();
        let new_id = new_street.id;
        new_street.set_start(&new_intersection);
        new_street.set_end(&self.intersection(&old_end).unwrap());
        new_intersection.add_outgoing_street(&new_street.id);

        if let Some(old_end) = self.intersection_mut(&old_end) as Option<&mut Intersection>
        {
            old_end.remove_connected_street(&street_id);
            old_end.add_incoming_street(&new_street.id);
        }

        self.add_street(new_street);
        self.add_intersection(new_intersection);

        self.update_intersection(&new_intersection_id);

        // Prevents visual glitches such as that the new street is not visible until the user moves the cursor
        self.update_street(&street_id);
        self.update_street(&new_id);

        self.update_intersection(&old_end);

        new_intersection_id
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

            if intersection.get_position().euclidean_distance(position) < offset {
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
                            intersections.push((street.id, intersection));
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
            if polygon.contains(&intersection.get_position()) {
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
                return Some(district.id);
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
                start_id = start.id;
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
                end_id = end.id;
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
                        intersected_streets.push(street.id);
                    }
                    _ => {}
                }
            }
        }

        intersected_streets
    }
}
