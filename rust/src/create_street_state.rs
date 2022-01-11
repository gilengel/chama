extern crate alloc;

use geo::Coordinate;
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{
    intersection::Intersection,
    map::{GetMut, Insert, Update},
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
}

impl<'a> State for CreateStreetState {
    fn mouse_down(&mut self, mouse_pos: Coordinate<f64>, button: u32, map: &mut Map) {
        // We only check for left click
        if button != 0 {
            return;
        }

        let start: &mut Intersection = map.get_mut(&self.temp_start).unwrap();
        start.set_position(mouse_pos);

        self.mouse_pressed = true;
    }

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map) {
        if !self.mouse_pressed {
            return;
        }

        let a: &mut Intersection = map.get_mut(&self.temp_end).unwrap();
        a.set_position(mouse_pos);
        
        map.update::<Street>(self.temp_street);
        
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, button: u32, map: &mut Map) {
        // Cancel creation of street with right mouse button click
        if button == 2 {
            self.mouse_pressed = false;
            return;
        }

        if self.mouse_pressed {
            self.temp_street = Uuid::new_v4();
            self.temp_start = Uuid::new_v4();
            self.temp_end = Uuid::new_v4();

            self.enter(map);
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
        let mut start = Intersection::default();
        start.id = self.temp_start;

        let mut end = Intersection::default();
        end.id = self.temp_end;

        let mut street = Street::default();
        street.id = self.temp_street;
        street.set_start(start.id, map);
        street.set_end(end.id, map);

        map.insert(start);
        map.insert(end);

        map.insert(street);
    }

    fn exit(&self, _map: &mut Map) {}
}
