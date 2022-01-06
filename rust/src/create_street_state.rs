use std::{cell::{RefCell, RefMut, Ref}, rc::Rc};

extern crate alloc;


use geo::{Coordinate};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{state::{State}, intersection::Intersection, street::Street, Map, Renderer};

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub struct CreateStreetState {
    mouse_pressed: bool,
    temp_street: Rc<RefCell<Street>>,
    temp_end: Rc<RefCell<Intersection>>,
}



fn option_borrow_mut<T>(a: &Option<Rc<RefCell<T>>>) -> RefMut<T> {
    a.as_ref().unwrap().as_ref().borrow_mut()
}

fn option_borrow<T>(a: &Option<Rc<RefCell<T>>>) -> Ref<T> {
    a.as_ref().unwrap().as_ref().borrow()
}

impl Default for CreateStreetState {
    fn default() -> CreateStreetState {
        CreateStreetState { 
            mouse_pressed: false,
            temp_street: Rc::new(RefCell::new(Street::default())),
            temp_end: Rc::new(RefCell::new(Intersection::default())),
        }
    }
}

impl CreateStreetState {
    pub fn new() -> Self {
        CreateStreetState::default()
    }
   
    
    fn add_temp_street_to_new_end(&mut self, _: &mut Map) {
        let temp_street = self.temp_street.as_ref().borrow();
        let mut end = option_borrow_mut(&temp_street.end);

        if !end.is_connected_to_street(Rc::clone(&self.temp_street)) {
            end.add_connected_street(Rc::clone(&self.temp_street));
        }
    }
    
    fn remove_temp_street_from_old_end(&mut self, map: &mut Map) {
        let temp_street = self.temp_street.as_ref().borrow();
        let mut end = option_borrow_mut(&temp_street.end);
        end.remove_connected_street(Rc::clone(&self.temp_street));

        if end.get_connected_streets().len() == 2 {
            let connected_streets = end.get_connected_streets();
            let mut street_1 = connected_streets[0].borrow_mut();
            let street_2 = &connected_streets[1].borrow();

            let diff = street_1.norm() - street_2.norm();
            if diff.x() < 0.001 && diff.y() < 0.001 {
                if let Some(_) = map.remove_street(Rc::clone(&connected_streets[1]))
                {
                    let end = street_2.end.as_ref().unwrap();
                    street_1.set_end(Rc::clone(&end));

                    map.remove_intersection(Rc::clone(&temp_street.end.as_ref().unwrap()));
                }
            }
        }
    }

    fn try_create_intersection_at_position(
        &mut self,
        position: Coordinate<f64>,
        map: &mut Map
    ) -> Option<Rc<RefCell<Intersection>>> {
        if let Some(intersected_street) = map.get_street_at_position(&position) {
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
            new_street.id = map.streets_length() as u32;

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

            map.add_street(Rc::clone(&new_street));
            map.add_intersection(Rc::clone(&intersection));

            return Some(intersection);
        }

        None
    }   
}

impl<'a> State for CreateStreetState {
    
    fn mouse_down(&mut self, x: u32, y: u32, button: u32, map: &mut Map) {
        // We only check for left click
        if button != 0 {
            return;
        }

        self.mouse_pressed = true;

        let position = Coordinate {
            x: x.into(),
            y: y.into(),
        };

        match map.get_intersection_at_position(&position, 100.0, &vec![]) {
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

        if let Some(intersection) = self.try_create_intersection_at_position(position, map) {
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

    fn mouse_move(&mut self, x: u32, y: u32, map: &mut Map) {
        let position: Coordinate<f64> = Coordinate {
            x: x.into(),
            y: y.into(),
        };
        if !self.mouse_pressed {
            return;
        }

        {
            match map.get_intersection_at_position(&position, 100.0, &vec![]) {
                Some(intersection) => {
                    self.remove_temp_street_from_old_end(map);

                    self.temp_street
                        .borrow_mut()
                        .set_end(Rc::clone(&intersection));
                    self.temp_street.borrow_mut().update_geometry();

                    self.add_temp_street_to_new_end(map);

                    return;
                }
                None => {
                    self.remove_temp_street_from_old_end(map);

                    {
                        let mut temp_street = self.temp_street.as_ref().borrow_mut();
                        temp_street.set_end(Rc::clone(&self.temp_end));
                    }

                    {
                        let mut end = self.temp_end.borrow_mut();
                        end.set_position(position);
                    }

                    self.add_temp_street_to_new_end(map);

                    {
                        let mut temp_street = self.temp_street.as_ref().borrow_mut();
                        temp_street.update_geometry();
                    }
                }
            }

            let mut temp_street = self.temp_street.as_ref().borrow_mut();
            match map.intersection_with_street(&temp_street) {
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

    fn mouse_up(&mut self, x: u32, y: u32, button: u32, map: &mut Map) {
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
            new_street.id = map.streets_length() as u32;

            let new_start = Rc::new(RefCell::new((*option_borrow(&temp_street.start)).clone()));
            if new_start.borrow().get_connected_streets().is_empty() {
                map.add_intersection(Rc::clone(&new_start));
            }
            new_street.start = Some(Rc::clone(&new_start));

            let new_end = Rc::new(RefCell::new((*option_borrow(&temp_street.end)).clone()));
            new_street.end = Some(Rc::clone(&new_end));
            if new_end.borrow().get_connected_streets().is_empty() {
                map.add_intersection(Rc::clone(&new_end));
            }

            let new_street = Rc::new(RefCell::new(new_street));

            {
                let mut new_start = new_start.as_ref().borrow_mut();
                new_start.add_connected_street(Rc::clone(&new_street));
                let mut new_end = new_end.as_ref().borrow_mut();
                new_end.add_connected_street(Rc::clone(&new_street));
            }

            map.add_street(Rc::clone(&new_street));
        }

        self.mouse_pressed = false;
    }

    fn render(&self, map: &Map, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        context.clear_rect(0.0, 0.0, map.width().into(), map.height().into());

        
        self.temp_street.as_ref().borrow().render(&context)?;

        log!("Die 10 Gebote");
        

        map.render(&context)?;
        Ok(())
    }

    

    fn update(&mut self) {

    }

    fn enter(&self) {

    }

    fn exit(&self) {
        
    }
}