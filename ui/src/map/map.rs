use geo::line_intersection::{line_intersection, LineIntersection};
use geo::prelude::{BoundingRect, Contains, EuclideanDistance};
use geo::{Coordinate, Line, LineString, Polygon, Rect};
use rust_editor::gizmo::{GetPosition, Id};
use rust_editor::interactive_element::{InteractiveElement, InteractiveElementState};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::cmp::Ordering;
use std::collections::hash_map::Keys;
use std::collections::HashMap;


use super::actions::street::delete::DeleteStreet;
use super::district::District;
use super::intersection::Intersection;
use super::street::Street;

#[derive(Serialize, Deserialize)]
pub struct Map {
    width: u32,
    height: u32,

    pub(crate) streets: HashMap<Uuid, Street>,
    pub(crate) intersections: HashMap<Uuid, Intersection>,
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

    pub fn intersections_mut(&mut self) -> &mut HashMap<Uuid, Intersection> {
        &mut self.intersections
    }

    pub fn intersections_keys<'a>(&'a self) -> Keys<'a, Uuid, Intersection> {
        self.intersections.keys()
    }

    pub fn districts(&self) -> &HashMap<Uuid, District> {
        &self.districts
    }

    pub fn districts_mut(&mut self) -> &mut HashMap<Uuid, District> {
        &mut self.districts
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

    pub fn line_intersection_with_streets(&self, line: &Line<f64>) -> Vec<(Uuid, Coordinate<f64>)> {
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

        intersections
    }

    pub fn line_intersection_with_street(
        &self,
        line: &Line<f64>,
    ) -> Option<(Uuid, Coordinate<f64>)> {
        let intersections = self.line_intersection_with_streets(line);

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

    pub fn remove_street(&mut self, id: &Uuid) -> Box<DeleteStreet> {
        Box::new(DeleteStreet::new(id.clone()))
    }

    /*
    pub fn remove_intersection(&mut self, id: &Uuid) -> Box<dyn Action<Map>> {
        let intersection = self.intersection(id).unwrap();

        Box::new(DeleteIntersection::new(intersection))
    }
    */

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
