use geo::line_intersection::{LineIntersection, line_intersection};
use geo::prelude::{BoundingRect, EuclideanDistance};
use geo::{Coordinate, LineString, Polygon, Rect, Line};

use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Deref;

use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::district::District;
use crate::intersection::Intersection;
use crate::street::Street;
use crate::Renderer;

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
    ) -> Result<(), JsValue> {
        
        for (_, district) in &self.districts {
            district.render(&context, additional_information_layer)?;
        }

        for (_, street) in &self.streets {
            street.render(&context, additional_information_layer)?;
        }

        for (_, intersection) in &self.intersections {
            intersection.render(&context, additional_information_layer)?;
        }

        Ok(())
    }
}

pub trait Get<T> {
    fn get(&self, id: &Uuid) -> Option<&'_ T>;
}

pub trait GetMut<T> {
    fn get_mut(&mut self, id: &Uuid) -> Option<&'_ mut T>;
}

impl Get<Street> for Map {
    fn get(&self, id: &Uuid) -> Option<&'_ Street> {
        if self.streets.contains_key(id) {
            return self.streets.get(id);
        }

        None
    }
}

impl Get<Intersection> for Map {
    fn get(&self, id: &Uuid) -> Option<&'_ Intersection> {
        if self.intersections.contains_key(id) {
            return self.intersections.get(id);
        }

        None
    }
}

impl GetMut<Street> for Map {
    fn get_mut(&mut self, id: &Uuid) -> Option<&'_ mut Street> {
        if self.streets.contains_key(id) {
            return self.streets.get_mut(id);
        }

        None
    }
}

impl GetMut<Intersection> for Map {
    fn get_mut(&mut self, id: &Uuid) -> Option<&'_ mut Intersection> {
        if self.intersections.contains_key(id) {
            return self.intersections.get_mut(id);
        }

        None
    }
}

impl GetMut<District> for Map {
    fn get_mut(&mut self, id: &Uuid) -> Option<&'_ mut District> {
        if self.districts.contains_key(id) {
            return self.districts.get_mut(id);
        }

        None
    }
}

pub trait Update<T> {
    fn update<S>(&mut self, id: &Uuid);
}

/*
pub trait Insert<T> {
    fn insert(&mut self, x: T);
}

impl Insert<Street> for Map {
    fn insert(&mut self, x: Street) {
        self.streets.insert(x.id, x);
    }
}

impl Insert<Intersection> for Map {
    fn insert(&mut self, x: Intersection) {
        self.intersections.insert(x.id, x);
    }
}
*/

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

    pub fn intersections(&self) -> &HashMap<Uuid, Intersection> {
        &self.intersections
    }

    pub fn streets(&self) -> &HashMap<Uuid, Street> {
        &self.streets
    }

    pub fn add_street(&mut self, street: Street) {
        self.streets.insert(street.id, street);
    }

    pub fn add_district(&mut self, district: District) {
        self.districts.insert(district.id, district);
    }

    pub fn add_intersection(&mut self, intersection: Intersection) {
        self.intersections.insert(intersection.id, intersection);

        self.update_bounding_box();
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
            self.bounding_box.set_min(self.bounding_box.min() - Coordinate { x: offset, y: offset});
            self.bounding_box.set_max(self.bounding_box.max() + Coordinate { x: offset, y: offset})
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
                //self.update_intersection(&start_id);
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
                //self.update_intersection(&end_id);
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

    pub fn streets_intersecting_ray(&self, ray_start_pos: &Coordinate<f64>, ray_direction: &Coordinate<f64>, ray_length: f64) -> Vec<Uuid> {
        let line = Line::new(*ray_start_pos, *ray_direction * ray_length);
        let mut intersected_streets = vec![];
        for (_, street) in &self.streets {
            let s : &Line<f64> = street.into();
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
