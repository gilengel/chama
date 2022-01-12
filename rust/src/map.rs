use geo::line_intersection::LineIntersection;
use geo::prelude::EuclideanDistance;
use geo::Coordinate;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;

use std::rc::Rc;
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::district::District;
use crate::intersection::{Intersection};
use crate::street::Street;
use crate::{Renderer};

pub struct Map {
    width: u32,
    height: u32,

    streets: HashMap<Uuid, Street>,
    intersections: HashMap<Uuid, Intersection>,
    districts: HashMap<Uuid, District>,
}

impl Default for Map {
    fn default() -> Map {
        Map {
            width: 1920,
            height: 800,
            streets: HashMap::new(),
            intersections: HashMap::new(),
            districts: HashMap::new(),
        }
    }
}

impl Renderer for Map {
    fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        for (_, district) in &self.districts {
            district.render(&context)?;
        }

        for (_, street) in &self.streets {
            street.render(&context)?;
        }

        for (_, intersection) in &self.intersections {
            intersection.render(&context)?;
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

pub trait Update<T> {
    fn update<S>(&mut self, id: &Uuid);
}

/*
impl<T> Update<Street> for T
    where
    T: GetMut<Street> + Get<Intersection>
{
    fn update<Street>(&mut self, id: &Uuid) {
        let street = self.get_mut(id).unwrap();
        let a = self.get(&street.start).unwrap();
        let b = self.get(&street.end).unwrap();

        street.line.start = a.get_position();
        street.line.end = b.get_position();

        street.update_geometry();
    }
}
*/

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

/*

pub trait Remove<T> {
    fn remove<S>(&mut self, id: &Uuid) -> Option<S>
}

impl Remove<Street> for Map {
    fn remove<Street>(&mut self, id: &Uuid) -> Option<Street> {
        self.streets.remove(id)
    }
}

impl Remove<Intersection> for Map {
    fn remove<Intersection>(&mut self, id: &Uuid) -> Option<Intersection> {
        self.streets.remove(&x.id)
    }
}
*/
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

    /*
    pub fn get_street(&self, id: Uuid) {
        if self.streets.contains_key(id) {
            return self.streets.get(id);
        }

        None
    }
    */

    pub fn add_street(&mut self, street: Street) {
        self.streets.insert(street.id, street);
    }

    pub fn add_district(&mut self, district: District) {
        self.districts.insert(district.id, district);
    }

    pub fn add_intersection(&mut self, intersection: Intersection) {
        self.intersections.insert(intersection.id, intersection);
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

    /*
    pub fn remove_street(&mut self, street: &Street) -> Option<bool> {
        if let Some((_key, _x)) = self.streets.remove_entry(&street.id) {
            todo!();
        }

        None

        match self.streets.iter().position(|(key, x)| key == street.id) {
            Some(index) => {
                let street_borrow = street.borrow();
                let start = street_borrow.start.as_ref().unwrap();
                let mut start_borrow = start.borrow_mut();
                start_borrow.remove_connected_street(Rc::clone(&street));
                start_borrow.reorder();
                if start_borrow.get_connected_streets().is_empty() {
                    self.remove_intersection(Rc::clone(&start));
                }

                let street_borrow = street.borrow();
                let end = street_borrow.end.as_ref().unwrap();
                let mut end_borrow = end.borrow_mut();
                end_borrow.remove_connected_street(Rc::clone(&street));
                end_borrow.reorder();
                if end_borrow.get_connected_streets().is_empty() {
                    self.remove_intersection(Rc::clone(&end));
                }

                self.streets.remove(index);

                Some(true)
            }
            None => None,
        }

    }
    */

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

    pub fn get_street_at_position(&self, position: &Coordinate<f64>, ignored_streets: &Vec<Uuid>) -> Option<Uuid> {
        for (id, street) in &self.streets {
            if ignored_streets.contains(id) { continue; }

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

    pub fn foo(&mut self, street_id: &Uuid, _start_id: &Uuid) {
        let a = self.streets.get_mut(street_id).unwrap();
        let b = self.intersections.get_mut(&a.start).unwrap();

        a.set_start(b);
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

    pub fn get_district_at_position(&self, position: &Coordinate<f64>) -> Option<&District> {
        for (_, district) in &self.districts {
            if district.is_point_on_district(position) {
                return Some(district);
            }
        }

        None
    }

    pub fn remove_street(&mut self, id: &Uuid) -> Option<Street> {
        if let Some(street) = self.streets.remove(id) {
            if let Some(start) = self.intersections.get_mut(&street.start) {
                start.remove_connected_street(id);
            }

            if let Some(end) = self.intersections.get_mut(&street.end) {
                end.remove_connected_street(id);
            }

            return Some(street);
        }

        None
    }

    pub fn remove_intersection(&mut self, id: &Uuid) -> Option<Intersection> {
        self.intersections.remove(id)
    }

    pub fn remove_district(&mut self, _district: Rc<RefCell<District>>) {
        /*
        if let Some(index) = self
            .districts
            .iter()
            .position(|i| Rc::ptr_eq(&i, &district))
        {
            self.districts.remove(index);
        }
        */
    }
}
