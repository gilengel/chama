use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::cmp::Ordering;
use std::rc::Rc;


use geo::Coordinate;
use geo::line_intersection::LineIntersection;

use geo::prelude::EuclideanDistance;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

mod street;
mod intersection;

use crate::street::Street;
use crate::intersection::Intersection;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub struct Editor {
    width: u32,
    height: u32,
    streets: Vec<Rc<RefCell<Street>>>,
    intersections: Vec<Rc<RefCell<Intersection>>>,

    temp_street: Rc<RefCell<Street>>,

    context: CanvasRenderingContext2d,

    mouse_pressed: bool,
}

fn get_canvas_and_context() -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("map_canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    Ok((canvas, context))
}

#[wasm_bindgen]
impl Editor {
    pub fn new() -> Editor {
        let (_, context) = get_canvas_and_context().unwrap();
        Editor {
            width: 1920,
            height: 800,
            streets: vec![],
            intersections: vec![],
            temp_street: Rc::new(RefCell::new(Street::default())),
            context,
            mouse_pressed: false,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn render(&self) -> Result<(), JsValue> {
        self.context
            .clear_rect(0.0, 0.0, self.width.into(), self.height.into());

        self.temp_street.as_ref().borrow().render(&self.context)?;

        for street in &self.streets {
            street.as_ref().borrow().render(&self.context)?;
        }

        for intersection in &self.intersections {
            intersection.as_ref().borrow().render(&self.context)?;
        }

        Ok(())
    }

    fn create_street_at_intersections(
        &self,
        start: Rc<RefCell<Intersection>>,
        end: Rc<RefCell<Intersection>>,
    ) -> Rc<RefCell<Street>> {
        let mut street = Street::default();

        street.set_start(start.clone());
        street.set_end(end.clone());
        street.id = self.streets.len() as u32;

        let street = Rc::new(RefCell::new(street));
        let mut start = start.as_ref().borrow_mut();
        start.connected_streets.push(Rc::clone(&street));

        let mut end = end.as_ref().borrow_mut();
        end.connected_streets.push(Rc::clone(&street));

        street
    }

    fn try_create_intersection_at_position(
        &mut self,
        position: Coordinate<f64>,
    ) -> Option<Rc<RefCell<Intersection>>> {
        // split street in case the new street starts on an existing one
        if let Some(intersected_street) = self.get_street_at_position(&position) {
            let mut street = intersected_street.as_ref().borrow_mut();

            let intersection = Rc::new(RefCell::new(Intersection {
                position: position,
                connected_streets: vec![Rc::clone(&intersected_street)]
            }));
            let new_street = self.create_street_at_intersections(
                Rc::clone(&intersection),
                Rc::clone(street.end.as_ref().unwrap()),
            );

            self.streets.push(Rc::clone(&new_street));

            street.end = Some(Rc::clone(&intersection));
            street.update_geometry();

            return Some(intersection);
        }

        None
    }

    pub fn mouse_down(&mut self, x: u32, y: u32) {
        let position = Coordinate {
            x: x.into(),
            y: y.into(),
        };

        {
            let mut temp_street = self.temp_street.as_ref().borrow_mut();
            temp_street.set_start(Rc::new(RefCell::new(Intersection {
                position,
                connected_streets: vec![Rc::clone(&self.temp_street)],
            })));
            temp_street.set_end(Rc::new(RefCell::new(Intersection {
                position,
                connected_streets: vec![Rc::clone(&self.temp_street)],
            })));
        }

        if let Some(intersection) = self.try_create_intersection_at_position(position) {
            let mut temp_street = self.temp_street.as_ref().borrow_mut();
            temp_street.set_start(Rc::clone(&intersection));
        }

        self.mouse_pressed = true
    }

    pub fn mouse_up(&mut self, _x: u32, _y: u32) {
        fn option_borrow_mut<T>(a: &Option<Rc<RefCell<T>>>) -> RefMut<T> {
            a.as_ref().unwrap().as_ref().borrow_mut()
        }

        fn option_borrow<T>(a: &Option<Rc<RefCell<T>>>) -> Ref<T> {
            a.as_ref().unwrap().as_ref().borrow()
        }

        if self.mouse_pressed {
            {
                let temp_street = self.temp_street.as_ref().borrow();
                let mut temp_start = option_borrow_mut(&temp_street.start);
                temp_start.remove_connected_street(Rc::clone(&self.temp_street));

                let mut temp_end = option_borrow_mut(&temp_street.end);
                temp_end.remove_connected_street(Rc::clone(&self.temp_street));
            }

            let temp_street = self.temp_street.as_ref().borrow();

            let mut new_street = temp_street.clone();
            new_street.id = self.streets.len() as u32;

            let new_start = Rc::new(RefCell::new((*option_borrow(&temp_street.start)).clone()));
            new_street.start = Some(Rc::clone(&new_start));

            let new_end = Rc::new(RefCell::new((*option_borrow(&temp_street.end)).clone()));
            new_street.end = Some(Rc::clone(&new_end));

            let new_street = Rc::new(RefCell::new(new_street));

            {
                let mut new_start = new_start.as_ref().borrow_mut();
                new_start.connected_streets.push(Rc::clone(&new_street));
                let mut new_end = new_end.as_ref().borrow_mut();
                new_end.connected_streets.push(Rc::clone(&new_street));
            }

            self.streets.push(Rc::clone(&new_street));

            self.intersections.push(Rc::clone(&new_start));
            self.intersections.push(Rc::clone(&new_end));

            
        }

        self.mouse_pressed = false;
    }

    pub fn mouse_move(&mut self, x: u32, y: u32) {
        let position = Coordinate {
            x: x.into(),
            y: y.into(),
        };
        if self.mouse_pressed {
            {
                let temp_street = self.temp_street.as_ref().borrow_mut();
                let mut end = temp_street.end.as_ref().unwrap().as_ref().borrow_mut();
                end.set_position(position);
            }
            {
                let mut temp_street = self.temp_street.as_ref().borrow_mut();
                temp_street.update_geometry();
            }

            let mut intersection: Option<Coordinate<f64>> = None;
            {
                let mut temp_street = self.temp_street.as_ref().borrow_mut();
                match self.intersection_with_street(&temp_street) {
                    Some(position) => {
                        let mut end = temp_street.end.as_ref().unwrap().as_ref().borrow_mut();
                        end.set_position(position);

                        intersection = Some(position.clone());
                    }
                    None => {}
                }
                temp_street.update_geometry();
            }

            if let Some(intersection) = intersection {
                if let Some(_intersection) = self.try_create_intersection_at_position(intersection)
                {
                    //let intersection = intersection.as_ref().borrow_mut();
                    //intersection.connected_streets.push(Rc::new(RefCell::new(self.temp_street)));
                }
            }
        }
    }

    fn get_street_at_position(&self, position: &Coordinate<f64>) -> Option<Rc<RefCell<Street>>> {
        for street in &self.streets {
            if street.as_ref().borrow().is_point_on_street(position) {
                return Some(street.clone());
            }
        }

        None
    }

    fn intersection_with_street(&self, street: &Street) -> Option<Coordinate<f64>> {
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
}
