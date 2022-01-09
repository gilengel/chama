use std::{
    cell::{RefCell},
    rc::Rc,
};

extern crate alloc;

use geo::Coordinate;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{
    
    intersection::{Direction, Intersection},
    state::State,
    street::Street,
    Map, Renderer,
};


pub struct CreateStreetState {
    mouse_pressed: bool,
    temp_street: Rc<RefCell<Street>>,
    temp_end: Rc<RefCell<Intersection>>,
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

    fn get_temp_end_without_borrowing_temp_street(
        &self,
        e: Rc<RefCell<Intersection>>,
        map: &Map,
    ) -> Rc<RefCell<Intersection>> {
        if Rc::ptr_eq(&self.temp_end, &e) {
            return Rc::clone(&self.temp_end);
        }

        let index = map
            .intersections()
            .iter()
            .position(|x| Rc::ptr_eq(x, &e))
            .unwrap();

        return Rc::clone(&map.intersections()[index]);
    }

    fn add_temp_street_to_new_end(&mut self, map: &mut Map) {
        {
            let temp_street = self.temp_street.as_ref().borrow();
            let mut end = temp_street.end.as_ref().unwrap().as_ref().borrow_mut();

            if !end.is_connected_to_street(Rc::clone(&self.temp_street)) {
                end.add_incoming_street(Rc::clone(&self.temp_street));
            }
        }

        //let mut end = None;
        {
            let t = self.temp_street.as_ref().borrow();
            Some(self.get_temp_end_without_borrowing_temp_street(
                Rc::clone(t.end.as_ref().unwrap()),
                map,
            ));
        }
    }

    fn remove_temp_street_from_old_end(&mut self, _map: &mut Map) {
        let temp_street = self.temp_street.as_ref().borrow();

        let mut end = temp_street.end.as_ref().unwrap().as_ref().borrow_mut();
        end.remove_connected_street(Rc::clone(&self.temp_street));
    }

    fn try_create_intersection_at_position(
        &mut self,
        position: Coordinate<f64>,
        map: &mut Map,
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
            intersection.add_incoming_street(Rc::clone(&intersected_street));

            let intersection = Rc::new(RefCell::new(intersection));
            old_street.set_end(Rc::clone(&intersection));

            let mut new_street = Street::default();
            new_street.set_start(Rc::clone(&intersection));
            new_street.set_end(Rc::clone(&old_end));
            new_street.id = map.streets_length() as u32;

            let new_street = Rc::new(RefCell::new(new_street));
            intersection
                .borrow_mut()
                .add_outgoing_street(Rc::clone(&new_street));
            old_end
                .as_ref()
                .borrow_mut()
                .add_incoming_street(Rc::clone(&new_street));

            {
                intersection
                    .borrow_mut()
                    .add_outgoing_street(Rc::clone(&self.temp_street));
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

                {
                    let mut temp_street = self.temp_street.as_ref().borrow_mut();
                    temp_street.set_start(Rc::clone(&intersection));
                    temp_street.set_end(Rc::clone(&self.temp_end));
                }

                intersection
                    .borrow_mut()
                    .add_outgoing_street(Rc::clone(&self.temp_street));
                return;
            }
            None => {
                let mut temp_street = self.temp_street.as_ref().borrow_mut();
                temp_street.set_start(Rc::new(RefCell::new(Intersection::new(
                    position,
                    vec![(Direction::Out, Rc::clone(&self.temp_street))],
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
                        .add_outgoing_street(Rc::clone(&self.temp_street));
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

                    {
                        self.temp_street
                            .borrow_mut()
                            .set_end(Rc::clone(&intersection));
                        self.temp_street.borrow_mut().update_geometry();
                    }

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
                    /*
                    {
                        let mut temp_street = self.temp_street.as_ref().borrow_mut();
                        temp_street.update_geometry();
                    }
                    */
                }
            }

            let mut temp_street = self.temp_street.as_ref().borrow_mut();

            match map.intersection_with_street(&temp_street) {
                Some(intersection) => {
                    let mut end = temp_street.end.as_ref().unwrap().as_ref().borrow_mut();
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
                let mut temp_start = temp_street.start.as_ref().unwrap().as_ref().borrow_mut();
                temp_start.remove_connected_street(Rc::clone(&self.temp_street));

                let mut temp_end = temp_street.end.as_ref().unwrap().as_ref().borrow_mut();
                temp_end.remove_connected_street(Rc::clone(&self.temp_street));
            }

            let temp_street = self.temp_street.as_ref().borrow();

            let mut new_street = temp_street.clone();
            new_street.id = map.streets_length() as u32;

            let new_street_rc = Rc::new(RefCell::new(new_street));

            if temp_street
                .start
                .as_ref()
                .unwrap()
                .as_ref()
                .borrow()
                .get_connected_streets()
                .is_empty()
            {
                let new_start = Rc::new(RefCell::new(
                    (*temp_street.start.as_ref().unwrap().as_ref().borrow_mut()).clone(),
                ));
                map.add_intersection(Rc::clone(&new_start));
                new_street_rc.borrow_mut().start = Some(Rc::clone(&new_start));

                new_start
                    .as_ref()
                    .borrow_mut()
                    .add_outgoing_street(Rc::clone(&new_street_rc));
            } else {
                let mut existing_start = temp_street.start.as_ref().unwrap().as_ref().borrow_mut();

                existing_start.add_outgoing_street(Rc::clone(&new_street_rc));
            }

            if temp_street
                .end
                .as_ref()
                .unwrap()
                .as_ref()
                .borrow()
                .get_connected_streets()
                .is_empty()
            {
                let new_end = Rc::new(RefCell::new(
                    (*&temp_street.end.as_ref().unwrap().as_ref().borrow_mut()).clone(),
                ));
                map.add_intersection(Rc::clone(&new_end));
                new_street_rc.borrow_mut().end = Some(Rc::clone(&new_end));

                new_end
                    .as_ref()
                    .borrow_mut()
                    .add_incoming_street(Rc::clone(&new_street_rc));
            } else {
                let mut existing_end = temp_street.end.as_ref().unwrap().as_ref().borrow_mut();
                existing_end.add_incoming_street(Rc::clone(&new_street_rc));
            }

            map.add_street(Rc::clone(&new_street_rc));

            let start = map
                .intersections()
                .iter()
                .position(|e| {
                    Rc::ptr_eq(e, &new_street_rc.as_ref().borrow().start.as_ref().unwrap())
                })
                .unwrap();
            map.intersections()[start].borrow_mut().reorder();
            let end = map
                .intersections()
                .iter()
                .position(|e| Rc::ptr_eq(e, &new_street_rc.as_ref().borrow().end.as_ref().unwrap()))
                .unwrap();
            map.intersections()[end].borrow_mut().reorder();

            //new_street_rc.borrow_mut().update_geometry();
        }

        self.mouse_pressed = false;
    }

    fn render(&self, map: &Map, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        context.clear_rect(0.0, 0.0, map.width().into(), map.height().into());

        if self.mouse_pressed {
            self.temp_street.as_ref().borrow().render(&context)?;
        }

        map.render(&context)?;

        context.set_fill_style(&"#FFFFFF".into());
        context.fill_text(
            format!(
                "intersections: {}, strreets: {}",
                map.intersections_length(),
                map.streets_length()
            )
            .as_str(),
            100.0,
            100.0,
        )?;

        Ok(())
    }

    fn update(&mut self) {}

    fn enter(&self) {}

    fn exit(&self) {}
}
