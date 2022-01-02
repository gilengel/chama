use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::cmp::Ordering;
use std::rc::Rc;

use geo::line_intersection::LineIntersection;
use geo::Coordinate;

use geo::prelude::EuclideanDistance;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

mod intersection;
mod street;

use crate::intersection::Intersection;
use crate::street::Street;

fn option_borrow_mut<T>(a: &Option<Rc<RefCell<T>>>) -> RefMut<T> {
    a.as_ref().unwrap().as_ref().borrow_mut()
}

fn option_borrow<T>(a: &Option<Rc<RefCell<T>>>) -> Ref<T> {
    a.as_ref().unwrap().as_ref().borrow()
}

#[wasm_bindgen]
pub struct Editor {
    width: u32,
    height: u32,
    streets: Vec<Rc<RefCell<Street>>>,
    intersections: Vec<Rc<RefCell<Intersection>>>,

    temp_street: Rc<RefCell<Street>>,
    temp_end: Rc<RefCell<Intersection>>,

    context: CanvasRenderingContext2d,

    mouse_pressed: bool,

    render_intersections: bool,
    render_streets: bool,
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
            temp_end: Rc::new(RefCell::new(Intersection::default())),
            context,
            mouse_pressed: false,
            render_intersections: true,
            render_streets: true,
        }
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

    pub fn set_render_intersections(&mut self, render: bool) {
        self.render_intersections = render;
    }

    pub fn set_render_streets(&mut self, render: bool) {
        self.render_streets = render;
    }

    pub fn render(&self) -> Result<(), JsValue> {
        self.context
            .clear_rect(0.0, 0.0, self.width.into(), self.height.into());

        if self.render_streets {
            self.temp_street.as_ref().borrow().render(&self.context)?;

            for street in &self.streets {
                street.as_ref().borrow().render(&self.context)?;
            }
        }

        if self.render_intersections {
            for intersection in &self.intersections {
                intersection.as_ref().borrow().render(&self.context)?;
            }
        }

        Ok(())
    }

    fn try_create_intersection_at_position(
        &mut self,
        position: Coordinate<f64>,
    ) -> Option<Rc<RefCell<Intersection>>> {
        if let Some(intersected_street) = self.get_street_at_position(&position) {
            let mut old_street = intersected_street.borrow_mut();
            let old_end = Rc::clone(&old_street.end.as_ref().unwrap());
            {
                let mut old_end = old_street.end.as_ref().unwrap().borrow_mut();
                old_end.remove_connected_street(Rc::clone(&intersected_street));
            }

            let mut intersection = Intersection::default();
            intersection.set_position(position);
            intersection.add_connected_street(Rc::clone(&intersected_street));

            let intersection = Rc::new(RefCell::new(intersection));
            old_street.set_end(Rc::clone(&intersection));

            let mut new_street = Street::default();
            new_street.set_start(Rc::clone(&intersection));
            new_street.set_end(Rc::clone(&old_end));
            new_street.id = self.streets.len() as u32;

            let new_street = Rc::new(RefCell::new(new_street));
            intersection
                .borrow_mut()
                .add_connected_street(Rc::clone(&new_street));
            old_end
                .as_ref()
                .borrow_mut()
                .add_connected_street(Rc::clone(&new_street));

            {
                intersection
                    .borrow_mut()
                    .add_connected_street(Rc::clone(&self.temp_street));
            }

            self.streets.push(Rc::clone(&new_street));
            self.intersections.push(Rc::clone(&intersection));

            return Some(intersection);
        }

        None
    }

    pub fn mouse_down(&mut self, x: u32, y: u32, button: u32) {
        // We only check for left click
        if button != 0 {
            return;
        }

        self.mouse_pressed = true;

        let position = Coordinate {
            x: x.into(),
            y: y.into(),
        };

        match self.get_intersection_at_position(&position, 100.0, &vec![]) {
            Some(intersection) => {
                self.temp_end.as_ref().borrow_mut().set_position(position);

                let mut temp_street = self.temp_street.as_ref().borrow_mut();
                temp_street.set_start(Rc::clone(&intersection));
                temp_street.set_end(Rc::clone(&self.temp_end));

                intersection
                    .borrow_mut()
                    .add_connected_street(Rc::clone(&self.temp_street));
                return;
            }
            None => {
                let mut temp_street = self.temp_street.as_ref().borrow_mut();
                temp_street.set_start(Rc::new(RefCell::new(Intersection::new(
                    position,
                    vec![Rc::clone(&self.temp_street)],
                ))));
            }
        }

        {
            self.temp_end.as_ref().borrow_mut().set_position(position);
            let mut temp_street = self.temp_street.as_ref().borrow_mut();
            temp_street.set_end(Rc::clone(&self.temp_end));
        }

        if let Some(intersection) = self.try_create_intersection_at_position(position) {
            {
                self.temp_street
                    .as_ref()
                    .borrow_mut()
                    .set_start(Rc::clone(&intersection));

                {
                    intersection
                        .borrow_mut()
                        .add_connected_street(Rc::clone(&self.temp_street));
                }
            }
        }
    }

    pub fn mouse_up(&mut self, x: u32, y: u32, button: u32) {
        let position = Coordinate {
            x: x.into(),
            y: y.into(),
        };
        // Cancel creation of street with right mouse button click
        if button == 2 {
            self.mouse_pressed = false;

            let mut temp_street = self.temp_street.as_ref().borrow_mut();
            temp_street.set_start_position(&position);
            temp_street.set_end_position(&position);
            temp_street.update_geometry();
            return;
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
            if new_start.borrow().get_connected_streets().is_empty() {
                self.intersections.push(Rc::clone(&new_start));
            }
            new_street.start = Some(Rc::clone(&new_start));

            let new_end = Rc::new(RefCell::new((*option_borrow(&temp_street.end)).clone()));
            new_street.end = Some(Rc::clone(&new_end));
            if new_end.borrow().get_connected_streets().is_empty() {
                self.intersections.push(Rc::clone(&new_end));
            }

            let new_street = Rc::new(RefCell::new(new_street));

            {
                let mut new_start = new_start.as_ref().borrow_mut();
                new_start.add_connected_street(Rc::clone(&new_street));
                let mut new_end = new_end.as_ref().borrow_mut();
                new_end.add_connected_street(Rc::clone(&new_street));
            }

            self.streets.push(Rc::clone(&new_street));
        }

        self.mouse_pressed = false;
    }

    fn remove_temp_street_from_old_end(&mut self) {
        let temp_street = self.temp_street.as_ref().borrow();
        let mut end = option_borrow_mut(&temp_street.end);
        end.remove_connected_street(Rc::clone(&self.temp_street));

        if end.get_connected_streets().len() == 2 {
            let connected_streets = end.get_connected_streets();
            let mut street_1 = connected_streets[0].borrow_mut();
            let street_2 = &connected_streets[1].borrow();

            let diff = street_1.norm() - street_2.norm();
            if diff.x() < 0.001 && diff.y() < 0.001 {
                if let Some(index) = self
                    .streets
                    .iter()
                    .position(|i| Rc::ptr_eq(&i, &connected_streets[1]))
                {
                    let end = street_2.end.as_ref().unwrap();
                    street_1.set_end(Rc::clone(&end));

                    self.streets.remove(index);

                    // remove intersection
                    if let Some(index) = self
                        .intersections
                        .iter()
                        .position(|i| Rc::ptr_eq(&i, &temp_street.end.as_ref().unwrap()))
                    {
                        self.intersections.remove(index);
                    }
                }
            }
        }
    }

    fn add_temp_street_to_new_end(&mut self) {
        let temp_street = self.temp_street.as_ref().borrow();
        let mut end = option_borrow_mut(&temp_street.end);

        if !end.is_connected_to_street(Rc::clone(&self.temp_street)) {
            end.add_connected_street(Rc::clone(&self.temp_street));
        }
    }

    pub fn mouse_move(&mut self, x: u32, y: u32) {
        let position: Coordinate<f64> = Coordinate {
            x: x.into(),
            y: y.into(),
        };
        if !self.mouse_pressed {
            return;
        }

        {
            match self.get_intersection_at_position(&position, 100.0, &vec![]) {
                Some(intersection) => {
                    self.remove_temp_street_from_old_end();

                    self.temp_street
                        .borrow_mut()
                        .set_end(Rc::clone(&intersection));
                    self.temp_street.borrow_mut().update_geometry();

                    self.add_temp_street_to_new_end();

                    return;
                }
                None => {
                    self.remove_temp_street_from_old_end();

                    {
                        let mut temp_street = self.temp_street.as_ref().borrow_mut();
                        temp_street.set_end(Rc::clone(&self.temp_end));
                    }

                    {
                        let mut end = self.temp_end.borrow_mut();
                        end.set_position(position);
                    }

                    self.add_temp_street_to_new_end();

                    {
                        let mut temp_street = self.temp_street.as_ref().borrow_mut();
                        temp_street.update_geometry();
                    }
                }
            }

            let mut temp_street = self.temp_street.as_ref().borrow_mut();
            match self.intersection_with_street(&temp_street) {
                Some(intersection) => {
                    let mut end = option_borrow_mut(&temp_street.end);
                    end.set_position(intersection);
                }
                None => {
                    self.temp_end.borrow_mut().set_position(position);
                    temp_street.set_end(Rc::clone(&self.temp_end));
                }
            }
            temp_street.update_geometry();
        }

    }

    fn get_intersection_at_position(
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
