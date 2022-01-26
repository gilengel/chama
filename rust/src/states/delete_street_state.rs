use geo::Coordinate;
use uuid::Uuid;

use crate::{
    interactive_element::{InteractiveElement, InteractiveElementState},
    state::State,
    Camera, Renderer, map::{map::{Map, InformationLayer}, intersection::Side, street::Street},
};

pub struct DeleteStreetState {
    hovered_streets: Option<Vec<Uuid>>,
}

impl DeleteStreetState {
    pub fn new() -> Self {
        DeleteStreetState {
            hovered_streets: None,
        }
    }

    fn clean_hovered_street_state(&self, map: &mut Map) {
        for (_, street) in map.streets_mut() {
            street.set_state(InteractiveElementState::Normal);
        }
    }

    fn connected_streets(&self, start: Uuid, map: &Map) -> Vec<Uuid> {
        let mut street = start;
        let mut forward = true;

        let mut streets: Vec<Uuid> = vec![];

        let mut side = Side::Left;

        let mut next = match map.street(&street).unwrap().get_next(side) {
            Some(id) => Some(id),
            None => None,
        };

        while next.is_some()
            && next.unwrap() != start
            && map
                .intersection(&map.street(&street).unwrap().end)
                .unwrap()
                .get_connected_streets()
                .len()
                == 2
        {
            streets.push(street);

            {
                let street = map.street(&street).unwrap();

                if forward {
                    next = match street.get_next(side) {
                        Some(id) => Some(id),
                        None => None,
                    };
                } else {
                    next = match street.get_previous(side) {
                        Some(id) => Some(id),
                        None => None,
                    };
                }

                if next.is_some()
                    && ((street.start == map.street(&next.unwrap()).unwrap().start)
                        || (street.end == map.street(&next.unwrap()).unwrap().end))
                {
                    forward = !forward;

                    side = match side {
                        Side::Left => Side::Right,
                        Side::Right => Side::Left,
                    }
                }
            }

            if let Some(next) = next {
                street = next;
            }
        }

        streets.push(street);

        streets
    }
}

impl Default for DeleteStreetState {
    fn default() -> DeleteStreetState {
        DeleteStreetState {
            hovered_streets: None,
        }
    }
}

impl State for DeleteStreetState {
    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, _: u32, _: &mut Map) {}

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map) {
        self.clean_hovered_street_state(map);

        if let Some(hovered_street) = map.get_street_at_position(&mouse_pos, &vec![]) {
            self.hovered_streets = Some(self.connected_streets(hovered_street, map));

            for street in self.hovered_streets.as_ref().unwrap() {
                if let Some(street) = map.street_mut(&street) as Option<&mut Street> {
                    street.set_state(InteractiveElementState::Hover)
                }
            }
        }
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, _: u32, map: &mut Map) {
        if let Some(hovered_streets) = &self.hovered_streets {
            for street in hovered_streets {
                map.remove_street(&street);
            }
            
        }
    }

    fn enter(&mut self, _map: &mut Map) {}

    fn exit(&self, map: &mut Map) {
        self.clean_hovered_street_state(map);
    }

    fn render(
        &self,
        map: &Map,
        context: &web_sys::CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
        camera: &Camera,
    ) -> Result<(), wasm_bindgen::JsValue> {
        map.render(context, additional_information_layer, camera)?;

        Ok(())
    }
}
