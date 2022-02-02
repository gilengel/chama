use geo::Coordinate;
use rust_editor::{
    actions::Action,
    camera::{Camera, Renderer},
    editor::EditorPlugin,
    interactive_element::{InteractiveElement, InteractiveElementState},
    system::System,
    InformationLayer,
};
use uuid::Uuid;

use crate::map::{intersection::Side, map::Map, street::Street};

pub struct DeleteStreetSystem {
    hovered_streets: Option<Vec<Uuid>>,
}

impl DeleteStreetSystem {
    pub fn new() -> Self {
        DeleteStreetSystem {
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

impl Default for DeleteStreetSystem {
    fn default() -> DeleteStreetSystem {
        DeleteStreetSystem {
            hovered_streets: None,
        }
    }
}

impl System<Map> for DeleteStreetSystem {
    fn mouse_down(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        _: u32,
        _: &mut Map,
        _plugins: &Vec<EditorPlugin<Map>>,
        _actions: &mut Vec<Box<dyn Action<Map>>>,
    ) {
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        map: &mut Map,
        _plugins: &Vec<EditorPlugin<Map>>,
        _actions: &mut Vec<Box<dyn Action<Map>>>,
    ) {
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

    fn mouse_up(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        _: u32,
        map: &mut Map,
        _plugins: &Vec<EditorPlugin<Map>>,
        _actions: &mut Vec<Box<dyn Action<Map>>>,
    ) {
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
        _plugins: &Vec<EditorPlugin<Map>>,
        camera: &Camera,
    ) -> Result<(), wasm_bindgen::JsValue> {
        map.render(context, additional_information_layer, camera)?;

        Ok(())
    }
}
