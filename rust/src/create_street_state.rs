extern crate alloc;

use geo::Coordinate;
use js_sys::Date;
use uuid::Uuid;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::CanvasRenderingContext2d;

use crate::{
    interactive_element::InteractiveElement,
    intersection::Intersection,
    log,
    map::{Get, GetMut, Insert, Update},
    state::State,
    street::Street,
    Map, Renderer,
};

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    }
}

pub struct CreateStreetState {
    mouse_pressed: bool,
    temp_street: Uuid,
    temp_start: Uuid,
    temp_end: Uuid,
}

impl Default for CreateStreetState {
    fn default() -> CreateStreetState {
        CreateStreetState {
            mouse_pressed: false,
            temp_street: Uuid::new_v4(),
            temp_start: Uuid::new_v4(),
            temp_end: Uuid::new_v4(),
        }
    }
}

impl CreateStreetState {
    pub fn new() -> Self {
        CreateStreetState::default()
    }

    /*
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
            let intersection =
                self.get_intersection_from_map(Rc::clone(t.end.as_ref().unwrap()), map);
            log!(":)")
        }
    }

    fn remove_temp_street_from_old_end(&mut self, _map: &mut Map) {
        let temp_street = self.temp_street.as_ref().borrow();

        let mut end = temp_street.end.as_ref().unwrap().as_ref().borrow_mut();
        end.remove_connected_street(Rc::clone(&self.temp_street));
        end.reorder();
    }

    fn try_create_intersection_at_position(
        &mut self,
        position: Coordinate<f64>,
        map: &mut Map,
    ) -> Option<Rc<RefCell<Intersection>>> {
        if let Some(intersected_street) = map.get_street_at_position(&position) {
            let mut old_street = intersected_street.borrow_mut();
            //let old_next_right = old_street.get_next(Side::Right);
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

            let old_next_left = old_street.get_next(Side::Left);
            let old_next_right = old_street.get_next(Side::Right);
            if let Some(old_next_left) = old_next_left {
                new_street.set_next(Side::Left, Some(Rc::clone(old_next_left)));
            }
            if let Some(old_next_right) = old_next_right {
                new_street.set_next(Side::Right, Some(Rc::clone(old_next_right)));
            }

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
    */

    fn create_new_start(&mut self) -> Intersection {
        self.temp_start = Uuid::new_v4();
        let mut start = Intersection::default();
        start.id = self.temp_start;

        start
    }

    fn create_new_end(&mut self) -> Intersection {
        self.temp_end = Uuid::new_v4();
        let mut end = Intersection::default();
        end.id = self.temp_end;

        end
    }
}

impl<'a> State for CreateStreetState {
    fn mouse_down(&mut self, mouse_pos: Coordinate<f64>, button: u32, map: &mut Map) {
        // We only check for left click
        if button != 0 {
            return;
        }

        self.mouse_pressed = true;

        self.temp_street = Uuid::new_v4();
        let mut street = Street::default();
        street.id = self.temp_street;

        let mut end = self.create_new_end();
        end.add_incoming_street(&self.temp_street);
        street.set_end(&end);
        map.insert(end);

        if let Some(hovered_intersection) =
            map.get_intersection_at_position(&mouse_pos, 100.0, &vec![])
        {
            self.temp_start = hovered_intersection;

            let hovered_intersection: &mut Intersection =
                map.get_mut(&hovered_intersection).unwrap();
            hovered_intersection.add_outgoing_street(&self.temp_street);
            street.set_start(&hovered_intersection);

            map.insert(street);

            return;
        }

        match map.get_street_at_position(&mouse_pos) {
            Some(hovered_street) => {
                let mut old_end: Option<Uuid> = None;
                let mut new_intersection = Intersection::default();
                self.temp_start = new_intersection.id;
                
                new_intersection.set_position(mouse_pos);
                street.set_start(&new_intersection);

                if let Some(hovered_street) = map.get_mut(&hovered_street) as Option<&mut Street> {
                    old_end = Some(hovered_street.end);
                    hovered_street.set_end(&new_intersection);
                }
                new_intersection.add_incoming_street(&hovered_street);
                new_intersection.add_outgoing_street(&self.temp_street);

                // The street from new intersection to the old end
                let mut new_street = Street::default();
                let new_id = new_street.id;
                new_street.set_start(&new_intersection);
                new_street.set_end(&map.get(&old_end.unwrap()).unwrap());
                new_intersection.add_outgoing_street(&new_street.id);

                if let Some(old_end) = map.get_mut(&old_end.unwrap()) as Option<&mut Intersection> {
                    old_end.remove_connected_street(&hovered_street);
                    old_end.add_incoming_street(&new_street.id);
                }

                

                map.insert(new_street);
                map.insert(new_intersection);

                // Prevents visual glitches such as that the new street is not visible until the user moves the cursor
                map.update_street(&hovered_street);
                map.update_street(&new_id);

                
            }
            None => {
                let mut start = self.create_new_start();
                start.set_position(mouse_pos);
                start.add_outgoing_street(&self.temp_street);
                street.set_start(&start);
                map.insert(start);
            }
        }

        map.insert(street);
    }

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map) {
        if !self.mouse_pressed {
            return;
        }

        let street: &Street = map.get(&self.temp_street).unwrap();
        let start = street.start;
        let mut end = street.end;

        match map.get_intersection_at_position(&mouse_pos, 100.0, &vec![start, self.temp_end]) {
            Some(intersection) => {
                if intersection != end {
                    if let Some(intersection) = map.intersection_mut(&self.temp_end) {
                        intersection.set_position(Coordinate {
                            x: -100.0,
                            y: -100.0,
                        });
                    }

                    map.intersection_mut(&end)
                        .unwrap()
                        .remove_connected_street(&self.temp_street);
                    map.intersection_mut(&intersection)
                        .unwrap()
                        .add_incoming_street(&self.temp_street);

                    let cloned_end = (map.get(&intersection).unwrap() as &Intersection).clone();
                    let street: &mut Street = map.get_mut(&self.temp_street).unwrap();
                    street.set_end(&cloned_end);
                }
            }
            None => {
                // Reset temp street end to the temp end intersection since it is the only one intersection allowed to follow
                // the mouse cursor. All other intersection are connected to at least one persisted street and moving one would
                // alter parts of the persisted map which is disallowed
                if end != self.temp_end {
                    map.intersection_mut(&end)
                        .unwrap()
                        .remove_connected_street(&self.temp_street);
                    map.update_intersection(&end);

                    end = self.temp_end;

                    let cloned_end = (map.get(&self.temp_end).unwrap() as &Intersection).clone();
                    let street: &mut Street = map.get_mut(&self.temp_street).unwrap();
                    street.set_end(&cloned_end);

                    map.intersection_mut(&self.temp_end)
                        .unwrap()
                        .add_incoming_street(&self.temp_street);
                }

                let intersection: &mut Intersection = map.get_mut(&end).unwrap();
                intersection.set_position(mouse_pos);
            }
        }

        map.update_street(&self.temp_street);
        map.update_intersection(&start);
        map.update_intersection(&end);
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, button: u32, map: &mut Map) {
        // Cancel creation of street with right mouse button click
        if button == 2 {
            self.mouse_pressed = false;
            return;
        }

        if self.mouse_pressed {

            //self.enter(map);
        }

        self.mouse_pressed = false;
    }

    fn render(&self, map: &Map, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        context.clear_rect(0.0, 0.0, map.width().into(), map.height().into());

        if self.mouse_pressed {
            //self.temp_street.as_ref().borrow().render(&context)?;
        }

        map.render(&context)?;

        context.set_fill_style(&"#FFFFFF".into());
        context.fill_text(
            format!(
                "intersections: {}, strreets: {}",
                map.intersections().len(),
                map.streets().len()
            )
            .as_str(),
            100.0,
            100.0,
        )?;

        Ok(())
    }

    fn enter(&self, map: &mut Map) {
        /*
        let mut start = Intersection::default();
        start.id = self.temp_start;

        let mut end = Intersection::default();
        end.id = self.temp_end;

        let mut street = Street::default();
        street.id = self.temp_street;
        street.set_start(&start);
        street.set_end(&end);

        start.add_outgoing_street(&street);
        end.add_outgoing_street(&street);

        map.insert(start);
        map.insert(end);

        map.insert(street);
        */
    }

    fn exit(&self, map: &mut Map) {
        let street = map.remove_street(&self.temp_street).unwrap();

        map.remove_intersection(&street.start);
        map.remove_intersection(&street.end);
    }
}
