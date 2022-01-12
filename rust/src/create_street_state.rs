extern crate alloc;



use geo::{
    line_intersection::{line_intersection, LineIntersection},
    Coordinate, Line, Point, prelude::EuclideanDistance,
};

use rand::{thread_rng, Rng};
use uuid::Uuid;
use wasm_bindgen::{JsValue};
use web_sys::CanvasRenderingContext2d;

use crate::{
    interactive_element::InteractiveElement,
    intersection::Intersection,
    map::{Get, GetMut, Insert},
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
    fn project_point_onto_middle_of_street(
        &self,
        point: Coordinate<f64>,
        street_id: &Uuid,
        map: &Map,
    ) -> Option<Coordinate<f64>> {
        let street: &Street = map.get(street_id).unwrap();

        let start = street.start();
        let end = street.end();

        let perp = street.perp();

        let line1 = Line::new(start, end);
        let line2 = Line::new(point + perp * -1000.0, point + perp * 1000.0);

        if let Some(intersection) = line_intersection(line1, line2) {
            match intersection {
                LineIntersection::SinglePoint {
                    intersection,
                    is_proper: _,
                } => {
                    return Some(intersection);
                }
                _ => return None,
            }
        }

        None
    }

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

    fn split_street(&mut self, pos: Coordinate<f64>, id: &Uuid, map: &mut Map) -> Uuid {
        let mut old_end: Option<Uuid> = None;
        let mut new_intersection = Intersection::default();
        let new_intersection_id = new_intersection.id;
        new_intersection.set_position(pos);

        if let Some(street) = map.get_mut(&id) as Option<&mut Street> {
            old_end = Some(street.end);
            street.set_end(&new_intersection);
        }

        new_intersection.add_incoming_street(&id);
        //new_intersection.add_incoming_street(&self.temp_street);

        // The street from new intersection to the old end
        let mut new_street = Street::default();
        let new_id = new_street.id;
        new_street.set_start(&new_intersection);
        new_street.set_end(&map.get(&old_end.unwrap()).unwrap());
        new_intersection.add_outgoing_street(&new_street.id);

        if let Some(old_end) = map.get_mut(&old_end.unwrap()) as Option<&mut Intersection> {
            old_end.remove_connected_street(&id);
            old_end.add_incoming_street(&new_street.id);
        }

        map.insert(new_street);
        map.insert(new_intersection);

        map.update_intersection(&new_intersection_id);

        // Prevents visual glitches such as that the new street is not visible until the user moves the cursor
        map.update_street(&id);
        map.update_street(&new_id);

        new_intersection_id
    }

    fn reset_temp_end(&mut self, map: &mut Map) {
        if let Some(intersection) = map.intersection_mut(&self.temp_end) {
            intersection.set_position(Coordinate {
                x: -100.0,
                y: -100.0,
            });
        }
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

        let mut rng = thread_rng();
        let r = rng.gen_range(0..255);
        let g = rng.gen_range(0..255);
        let b = rng.gen_range(0..255);
        street.style.normal.border_color = format!("rgb({},{},{})", r, g, b).to_string();
        street.id = self.temp_street;

        let mut end = self.create_new_end();
        end.add_incoming_street(&self.temp_street);
        end.set_position(mouse_pos - Coordinate { x: 0.0, y: 100.0 });
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

        match map.get_street_at_position(&mouse_pos, &vec![]) {
            Some(hovered_street) => {
                let mouse_pos = self
                    .project_point_onto_middle_of_street(mouse_pos, &hovered_street, map)
                    .unwrap();
                self.temp_start = self.split_street(mouse_pos, &hovered_street, map);
                let start: &mut Intersection = map.get_mut(&self.temp_start).unwrap();

                start.add_outgoing_street(&street.id);
                street.set_start(start);
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
                    self.reset_temp_end(map);

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
                match map.get_street_at_position(&mouse_pos, &vec![self.temp_street]) {
                    Some(hovered_street) => {                        
                        let pos = self
                            .project_point_onto_middle_of_street(mouse_pos, &hovered_street, map)
                            .unwrap();
                        
                        let p: Point<f64> = pos.into();
                        let s: Point<f64> = map.intersection(&self.temp_start).unwrap().get_position().into();
                        if p.euclidean_distance(&s) > 10.0 {
                            self.split_street(pos, &hovered_street, map);
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

                            let cloned_end =
                                (map.get(&self.temp_end).unwrap() as &Intersection).clone();
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
            }
        }

        map.update_street(&self.temp_street);
        map.update_intersection(&start);
        map.update_intersection(&end);
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, button: u32, _map: &mut Map) {
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

    fn enter(&self, _map: &mut Map) {
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
