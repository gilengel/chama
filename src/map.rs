use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use geo::Coordinate;
use geo::line_intersection::LineIntersection;
use geo::prelude::EuclideanDistance;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::Renderer;
use crate::intersection::Intersection;
use crate::street::Street;

pub struct Map {
    width: u32,
    height: u32,

    pub streets: Vec<Rc<RefCell<Street>>>,
    pub intersections: Vec<Rc<RefCell<Intersection>>>,
}

impl Map {
    pub fn new() -> Map {
        Map::default()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn intersections_length(&self) -> usize {
        self.intersections.len()
    }

    pub fn streets_length(&self) -> usize {
        self.streets.len()
    }
}

impl Default for Map {
    fn default() -> Map {
        Map {
            width: 1920,
            height: 800,
            streets: vec![],
            intersections: vec![],
        }
    }
}

impl Renderer for Map {
    fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        //context.clear_rect(0.0, 0.0, self.width.into(), self.height.into());

        //if self.render_streets {
        //    self.temp_street.as_ref().borrow().render(&self.context)?;

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

    pub fn get_street_at_position(&self, position: &Coordinate<f64>) -> Option<Rc<RefCell<Street>>> {
        for street in &self.streets {
            if street.as_ref().borrow().is_point_on_street(position) {
                return Some(street.clone());
            }
        }

        None
    }    
}

