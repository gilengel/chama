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
use crate::intersection::Intersection;
use crate::street::Street;
use crate::Renderer;

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

pub trait Get<'a, T> {
    fn get(&'a self, id: Uuid) -> &'a T;
}

pub trait GetMut<'a, T> {
    fn get_mut(&'a mut self, id: Uuid) -> &'a mut T;
}

impl Get<'_, Street> for Map {
    fn get<'a>(&'a self, id: Uuid) -> &'a Street {
        self.streets.get(&id).unwrap()
    }
}

impl Get<'_, Intersection> for Map {
    fn get<'a>(&'a self, id: Uuid) -> &'a Intersection {
        self.intersections.get(&id).unwrap()
    }   
}

impl GetMut<'_, Street> for Map {
    fn get_mut<'a>(&'a mut self, id: Uuid) -> &'a mut Street {
        self.streets.get_mut(&id).unwrap()
    }
}

impl GetMut<'_, Intersection> for Map {
    fn get_mut<'a>(&'a mut self, id: Uuid) -> &'a mut Intersection {
        self.intersections.get_mut(&id).unwrap()
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

    pub fn streets (&self) -> &HashMap<Uuid, Street> {
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
    }

    pub fn remove_street(&mut self, street: &Street) -> Option<bool> {
        if let Some((key, x)) = self.streets.remove_entry(&street.id) {
            todo!();
        }

        None
        /*
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
        */
    }

    pub fn remove_intersection(&mut self, intersection: &Intersection) {
        self.intersections.remove(&intersection.id);
    }

    pub fn get_intersection_at_position(
        &self,
        position: &Coordinate<f64>,
        offset: f64,
        ignored_intersections: &Vec<Uuid>,
    ) -> Option<&Intersection> {
        for (id, intersection) in &self.intersections {
            if ignored_intersections.into_iter().any(|e| e == id) {
                continue;
            }

            if intersection.get_position().euclidean_distance(position) < offset {
                return Some(&intersection);
            }
        }

        None
    }

    pub fn intersection_with_street(&self, street: &Street) -> Option<Coordinate<f64>> {
        let mut intersections = vec![];

        for (_, another_street) in &self.streets {
            if let Some(line_intersection) =
                street.intersect_with_street(another_street)
            {
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
    ) -> Option<&Street> {
        for (_, street) in &self.streets {
            if street.is_point_on_street(position) {
                return Some(street);
            }
        }

        None
    }

    pub fn get_nearest_street_to_position(
        &self,
        position: &Coordinate<f64>,
    ) -> Option<&Street> {
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

    pub fn get_district_at_position(
        &self,
        position: &Coordinate<f64>,
    ) -> Option<&District> {
        for (_, district) in &self.districts {
            if district.is_point_on_district(position) {
                return Some(district);
            }
        }

        None
    }

    pub fn remove_district(&mut self, district: Rc<RefCell<District>>) {
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
