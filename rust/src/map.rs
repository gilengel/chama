use geo::line_intersection::LineIntersection;
use geo::prelude::EuclideanDistance;
use geo::Coordinate;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::district::District;
use crate::intersection::Intersection;
use crate::street::Street;
use crate::Renderer;

pub struct Map {
    width: u32,
    height: u32,

    streets: Vec<Rc<RefCell<Street>>>,
    intersections: Vec<Rc<RefCell<Intersection>>>,
    districts: Vec<Rc<RefCell<District>>>
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

    pub fn intersections(&self) -> &Vec<Rc<RefCell<Intersection>>> {
        &self.intersections
    }

    pub fn intersections_length(&self) -> usize {
        self.intersections.len()
    }

    pub fn streets_length(&self) -> usize {
        self.streets.len()
    }

    pub fn add_street(&mut self, street: Rc<RefCell<Street>>) {
        self.streets.push(street);
    }

    pub fn add_district(&mut self, district: Rc<RefCell<District>>) {
        self.districts.push(district);
    }

    pub fn remove_street(&mut self, street: Rc<RefCell<Street>>) -> Option<bool> {
        match self.streets.iter().position(|i| Rc::ptr_eq(&i, &street)) {
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

    pub fn add_intersection(&mut self, intersection: Rc<RefCell<Intersection>>) {
        self.intersections.push(intersection);
    }

    pub fn remove_intersection(&mut self, intersection: Rc<RefCell<Intersection>>) {
        if let Some(index) = self
            .intersections
            .iter()
            .position(|i| Rc::ptr_eq(&i, &intersection))
        {
            self.intersections.remove(index);
        }
    }
}

impl Default for Map {
    fn default() -> Map {
        Map {
            width: 1920,
            height: 800,
            streets: vec![],
            intersections: vec![],
            districts: vec![]
        }
    }
}

impl Renderer for Map {
    fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        //context.clear_rect(0.0, 0.0, self.width.into(), self.height.into());

        //if self.render_streets {
        //    self.temp_street.as_ref().borrow().render(&self.context)?;

        for district in &self.districts {
            district.as_ref().borrow().render(&context)?;
        }

        for street in &self.streets {
            street.as_ref().borrow().render(&context)?;
        }
        //}

        //if self.render_intersections {
        for intersection in &self.intersections {
            intersection.as_ref().borrow().render(&context)?;
        }
        //}



        Ok(())
    }
}

impl Map {
    pub fn get_intersection_at_position(
        &self,
        position: &Coordinate<f64>,
        offset: f64,
        ignored_intersections: &Vec<Rc<RefCell<Intersection>>>,
    ) -> Option<Rc<RefCell<Intersection>>> {
        for intersection in &self.intersections {
            if ignored_intersections
                .into_iter()
                .any(|e| Rc::ptr_eq(e, intersection))
            {
                continue;
            }

            let a = intersection.as_ref().borrow();
            let intersection_pos = a.get_position();
            if intersection_pos.euclidean_distance(position) < offset {
                return Some(Rc::clone(&intersection));
            }
        }

        None
    }

    pub fn intersection_with_street(&self, street: &Street) -> Option<Coordinate<f64>> {
        let mut intersections = vec![];

        for another_street in &self.streets {
            if let Some(line_intersection) =
                street.intersect_with_street(&another_street.as_ref().borrow())
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
    ) -> Option<Rc<RefCell<Street>>> {
        for street in &self.streets {
            if street.as_ref().borrow().is_point_on_street(position) {
                return Some(Rc::clone(street));
            }
        }

        None
    }
}
