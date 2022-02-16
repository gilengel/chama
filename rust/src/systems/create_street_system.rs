extern crate alloc;

use std::collections::HashMap;

use geo::{
    line_intersection::{line_intersection, LineIntersection},
    prelude::EuclideanDistance,
    Coordinate, Line, Point,
};

use rand::{thread_rng, Rng};
use rust_editor::{
    gizmo::{GetPosition, Id, SetId, SetPosition},
    system::System,
    InformationLayer, plugins::plugin::{PluginWithOptions},
};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{map::{
    intersection::{Direction, Intersection},
    map::Map,
    street::Street,
}, Modes};

pub struct CreateStreetSystem {
    mouse_pressed: bool,
    temp_street: Uuid,
    temp_start: Uuid,
    temp_end: Uuid,
}

impl Default for CreateStreetSystem {
    fn default() -> CreateStreetSystem {
        CreateStreetSystem {
            mouse_pressed: false,
            temp_street: Uuid::new_v4(),
            temp_start: Uuid::new_v4(),
            temp_end: Uuid::new_v4(),
        }
    }
}

impl CreateStreetSystem {
    pub fn new() -> Self {
        CreateStreetSystem::default()
    }
    fn project_point_onto_middle_of_street(
        &self,
        point: Coordinate<f64>,
        street_id: &Uuid,
        map: &Map,
    ) -> Coordinate<f64> {
        let street: &Street = map.street(street_id).unwrap();

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
                    return intersection;
                }
                _ => return point,
            }
        }

        point
    }

    fn create_new_start(&mut self, position: &Coordinate<f64>) -> Intersection {
        self.temp_start = Uuid::new_v4();
        let mut start = Intersection::default();
        start.set_id(self.temp_start);
        start.set_position(*position);
        start.add_outgoing_street(&self.temp_street);

        start
    }

    fn create_new_end(&mut self, position: &Coordinate<f64>) -> Intersection {
        self.temp_end = Uuid::new_v4();
        let mut end = Intersection::default();
        end.set_id(self.temp_end);
        end.set_position(*position);
        end.add_incoming_street(&self.temp_street);

        end
    }

    fn split_street(
        &mut self,
        pos: Coordinate<f64>,
        street_id: &Uuid,
        map: &mut Map,
    ) -> Option<Uuid> {
        let mut old_end: Option<Uuid> = None;
        let mut new_intersection = Intersection::default();
        let new_intersection_id = new_intersection.id();
        new_intersection.set_position(pos);

        if let Some(street) = map.street_mut(&street_id) as Option<&mut Street> {
            old_end = Some(street.end);
            street.set_end(&new_intersection);
        }

        new_intersection.add_incoming_street(&street_id);

        // The street from new intersection to the old end
        let mut new_street = Street::default();
        let new_id = new_street.id();
        new_street.set_start(&new_intersection);
        new_street.set_end(&map.intersection(&old_end.unwrap()).unwrap());
        new_intersection.add_outgoing_street(&new_street.id());

        if let Some(old_end) = map.intersection_mut(&old_end.unwrap()) as Option<&mut Intersection>
        {
            old_end.remove_connected_street(&street_id);
            old_end.add_incoming_street(&new_street.id());
        }

        map.add_street(new_street);
        map.add_intersection(new_intersection);

        map.update_intersection(&new_intersection_id);

        // Prevents visual glitches such as that the new street is not visible until the user moves the cursor
        map.update_street(&street_id);
        map.update_street(&new_id);

        map.update_intersection(&old_end.unwrap());
        Some(new_intersection_id)
    }

    fn is_splitting_street_allowed(&mut self, pos: Coordinate<f64>, map: &&mut Map) -> bool {
        let p: Point<f64> = pos.into();
        let s: Point<f64> = map
            .intersection(&self.temp_start)
            .unwrap()
            .position()
            .into();

        p.euclidean_distance(&s) > 10.0
    }

    fn pair_of_connected_streets_of_intersection(
        &self,
        intersection_id: Uuid,
        map: &Map,
    ) -> (Uuid, Uuid) {
        let intersection = map.intersection(&intersection_id).unwrap();

        let x = intersection.get_connected_streets()[0];
        let y = intersection.get_connected_streets()[1];

        let incoming_street_id = if x.0 == Direction::In { x.1 } else { y.1 };
        let outgoing_street_id = if x.0 == Direction::In { y.1 } else { x.1 };

        (incoming_street_id, outgoing_street_id)
    }

    fn is_intersection_deletable(&self, intersection_id: Uuid, map: &Map) -> bool {
        let intersection = map.intersection(&intersection_id).unwrap();

        let connected_streets = intersection.get_connected_streets();
        if connected_streets.len() != 2 {
            return false;
        }

        // We only have a possibly deletable intersaction if both of the connected streets
        // have different directions. Otherwise the intersection was manually created and therefore
        // is not deletable.
        if connected_streets[0].0 == connected_streets[1].0 {
            return false;
        }

        let (incoming_street_id, outgoing_street_id) =
            self.pair_of_connected_streets_of_intersection(intersection_id, map);

        let norm_1 = map.street(&incoming_street_id).unwrap().norm();
        let norm_2 = map.street(&outgoing_street_id).unwrap().norm();
        let diff = norm_1 - norm_2;

        return diff.x.abs() < 0.000001 && diff.y.abs() < 0.000001;
    }

    fn delete_intersection(&self, intersection_id: Uuid, map: &mut Map) {
        let (incoming_street_id, outgoing_street_id) =
            self.pair_of_connected_streets_of_intersection(intersection_id, map);

        let old_end = map.street(&outgoing_street_id).unwrap().end;

        map.intersection_mut(&old_end)
            .unwrap()
            .add_incoming_street(&incoming_street_id);

        let old_end = map.intersection(&old_end).unwrap().clone();
        map.street_mut(&incoming_street_id)
            .unwrap()
            .set_end(&old_end);

        map.remove_street(&outgoing_street_id);
        map.remove_intersection(&intersection_id);
    }

    fn switch_intersections(
        &self,
        old_intersection_id: &Uuid,
        new_intersection_id: &Uuid,
        map: &mut Map,
    ) {
        map.intersection_mut(&old_intersection_id)
            .unwrap()
            .remove_connected_street(&self.temp_street);
        map.update_intersection(old_intersection_id);

        if self.is_intersection_deletable(*old_intersection_id, map) {
            self.delete_intersection(*old_intersection_id, map);
        }

        let cloned_end = map.intersection(&new_intersection_id).unwrap().clone();
        map.street_mut(&self.temp_street)
            .unwrap()
            .set_end(&cloned_end);

        map.intersection_mut(&new_intersection_id)
            .unwrap()
            .add_incoming_street(&self.temp_street);

        map.update_intersection(new_intersection_id);
    }
}

impl<'a> System<Map, Modes> for CreateStreetSystem {
    fn mouse_down(
        &mut self,
        mouse_pos: Coordinate<f64>,
        button: u32,
        map: &mut Map,
        

       _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>
    ) {
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
        street.style.normal.border_width = 0;
        street.set_id(self.temp_street);

        let end = self.create_new_end(&mouse_pos);
        street.set_end(&end);
        map.add_intersection(end);

        if let Some(hovered_intersection) =
            map.get_intersection_at_position(&mouse_pos, 100.0, &vec![self.temp_end])
        {
            self.temp_start = hovered_intersection;

            let hovered_intersection = map.intersection_mut(&hovered_intersection).unwrap();
            hovered_intersection.add_outgoing_street(&self.temp_street);
            street.set_start(&hovered_intersection);

            map.add_street(street);

            return;
        }

        match map.get_street_at_position(&mouse_pos, &vec![]) {
            Some(hovered_street) => {
                let mouse_pos =
                    self.project_point_onto_middle_of_street(mouse_pos, &hovered_street, map);

                self.temp_start = self.split_street(mouse_pos, &hovered_street, map).unwrap();

                let start: &mut Intersection = map.intersection_mut(&self.temp_start).unwrap();

                start.add_outgoing_street(&street.id());
                street.set_start(start);
            }
            None => {
                let start = self.create_new_start(&mouse_pos);
                street.set_start(&start);
                map.add_intersection(start);
            }
        }

        map.add_street(street);
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        map: &mut Map,        

       _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>
    ) {
        if !self.mouse_pressed {
            return;
        }

        let street = map.street_mut(&self.temp_street).unwrap();
        let current_start = street.start;
        let current_end = street.end;

        let current_start_pos: Point<f64> =
            map.intersection(&current_end).unwrap().position().into();
        let current_end_pos: Point<f64> =
            map.intersection(&current_start).unwrap().position().into();

        // Only update the position of the temp end if it is to close to the temp start. This prevents
        // that the routine will set the temp end to the wrong intersection and result in visual issues.
        if current_start_pos.euclidean_distance(&current_end_pos) < 100.0 {
            let intersection: &mut Intersection = map.intersection_mut(&current_end).unwrap();
            intersection.set_position(mouse_pos);

            let street = map.street(&self.temp_street).unwrap();
            let start = street.start;
            let end = street.end;
            map.update_intersection(&end);
            map.update_intersection(&start);

            return;
        }

        match map.get_intersection_at_position(&mouse_pos, 50.0, &vec![self.temp_end]) {
            Some(intersection) => {
                if intersection != current_end {
                    self.switch_intersections(&current_end, &intersection, map);
                }
            }
            None => {
                match map.get_street_at_position(&mouse_pos, &vec![self.temp_street]) {
                    Some(hovered_street) => {
                        let pos = self.project_point_onto_middle_of_street(
                            mouse_pos,
                            &hovered_street,
                            map,
                        );

                        if self.is_splitting_street_allowed(pos, &map) {
                            let new_intersection =
                                self.split_street(mouse_pos, &hovered_street, map).unwrap();
                            self.switch_intersections(&current_end, &new_intersection, map);
                        }
                    }
                    None => {
                        // Reset temp street end to the temp end intersection since it is the only one intersection allowed to follow
                        // the mouse cursor. All other intersection are connected to at least one persisted street and moving one would
                        // alter parts of the persisted map which is disallowed
                        if current_end != self.temp_end {
                            self.switch_intersections(&current_end, &self.temp_end, map);
                        } else {
                            let intersection: &mut Intersection =
                                map.intersection_mut(&current_end).unwrap();
                            intersection.set_position(mouse_pos);
                            map.update_bounding_box();
                        }
                    }
                }
            }
        }

        let street = map.street(&self.temp_street).unwrap();
        let start = street.start;
        let end = street.end;
        map.update_intersection(&end);
        map.update_intersection(&start);
    }

    fn mouse_up(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        button: u32,
        _map: &mut Map,        

       _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>
    ) {
        // Cancel creation of street with right mouse button click
        if button == 2 {
            self.mouse_pressed = false;
            return;
        }

        self.mouse_pressed = false;

        // invalidate temp_street to prevent deletion if the user switches to another state
        // This does not prevent the creation of new streets since temp_street will be set
        // on the mouse down event
        self.temp_street = Uuid::default();
    }

    fn render(
        &self,
        _map: &Map,
        _context: &CanvasRenderingContext2d,
        _additional_information_layer: &Vec<InformationLayer>,
        _plugins: &HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>
        
    ) -> Result<(), JsValue> {
        Ok(())
    }

    fn exit(&self, data: &mut Map, _plugins: HashMap<&'static str, &mut Box<dyn PluginWithOptions<Map, Modes>>>) {
        if let Some(intersection) = data.intersection(&self.temp_end) {
            if intersection.get_connected_streets().is_empty() {
                data.remove_intersection(&self.temp_end);
            }
        }
    }
}
