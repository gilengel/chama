use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;

use std::rc::Rc;


use geo::Coordinate;


use map::Map;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

mod intersection;
mod street;
mod map;

use crate::intersection::Intersection;
use crate::street::Street;

fn option_borrow_mut<T>(a: &Option<Rc<RefCell<T>>>) -> RefMut<T> {
    a.as_ref().unwrap().as_ref().borrow_mut()
}

fn option_borrow<T>(a: &Option<Rc<RefCell<T>>>) -> Ref<T> {
    a.as_ref().unwrap().as_ref().borrow()
}

trait Renderer {
    fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue>;
}






#[wasm_bindgen]
pub struct Editor {
    temp_street: Rc<RefCell<Street>>,
    temp_end: Rc<RefCell<Intersection>>,

    context: CanvasRenderingContext2d,

    mouse_pressed: bool,

    render_intersections: bool,
    render_streets: bool,
    map: Map,
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
            temp_street: Rc::new(RefCell::new(Street::default())),
            temp_end: Rc::new(RefCell::new(Intersection::default())),
            context,
            mouse_pressed: false,
            render_intersections: true,
            render_streets: true,
            map: Map::default(),
        }
    }

    pub fn width(&self) -> u32 {
        self.map.width()
    }

    pub fn height(&self) -> u32 {
        self.map.height()
    }

    pub fn intersections_length(&self) -> usize {
        self.map.intersections_length()
    }

    pub fn streets_length(&self) -> usize {
        self.map.streets_length()
    }

    pub fn set_render_intersections(&mut self, render: bool) {
        self.render_intersections = render;
    }

    pub fn set_render_streets(&mut self, render: bool) {
        self.render_streets = render;
    }

    pub fn render(&self) -> Result<(), JsValue> {       
        self.context.clear_rect(0.0, 0.0, self.map.width().into(), self.map.height().into());

        if self.render_streets {
            self.temp_street.as_ref().borrow().render(&self.context)?;
        }

        self.map.render(&self.context)?;

        Ok(())
    }
    
    fn try_create_intersection_at_position(
        &mut self,
        position: Coordinate<f64>,
    ) -> Option<Rc<RefCell<Intersection>>> {
        if let Some(intersected_street) = self.map.get_street_at_position(&position) {
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
            new_street.id = self.map.streets_length() as u32;

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

            self.map.add_street(Rc::clone(&new_street));
            self.map.add_intersection(Rc::clone(&intersection));

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

        match self.map.get_intersection_at_position(&position, 100.0, &vec![]) {
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
            new_street.id = self.map.streets_length() as u32;

            let new_start = Rc::new(RefCell::new((*option_borrow(&temp_street.start)).clone()));
            if new_start.borrow().get_connected_streets().is_empty() {
                self.map.add_intersection(Rc::clone(&new_start));
            }
            new_street.start = Some(Rc::clone(&new_start));

            let new_end = Rc::new(RefCell::new((*option_borrow(&temp_street.end)).clone()));
            new_street.end = Some(Rc::clone(&new_end));
            if new_end.borrow().get_connected_streets().is_empty() {
                self.map.add_intersection(Rc::clone(&new_end));
            }

            let new_street = Rc::new(RefCell::new(new_street));

            {
                let mut new_start = new_start.as_ref().borrow_mut();
                new_start.add_connected_street(Rc::clone(&new_street));
                let mut new_end = new_end.as_ref().borrow_mut();
                new_end.add_connected_street(Rc::clone(&new_street));
            }

            self.map.add_street(Rc::clone(&new_street));
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
                if let Some(_) = self.map.remove_street(Rc::clone(&connected_streets[1]))
                {
                    let end = street_2.end.as_ref().unwrap();
                    street_1.set_end(Rc::clone(&end));

                    self.map.remove_intersection(Rc::clone(&temp_street.end.as_ref().unwrap()));
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
            match self.map.get_intersection_at_position(&position, 100.0, &vec![]) {
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
            match self.map.intersection_with_street(&temp_street) {
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
}