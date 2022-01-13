use std::{cell::RefCell, rc::Rc};

use geo::Coordinate;
use uuid::Uuid;

use crate::{
    interactive_element::{InteractiveElement, InteractiveElementState},
    map::{GetMut, Map},
    state::State,
    street::Street,
    Renderer,
};

pub struct DeleteStreetState {
    hovered_street: Option<Uuid>,
}

impl DeleteStreetState {
    pub fn new() -> Self {
        DeleteStreetState {
            hovered_street: None,
        }
    }
}

impl Default for DeleteStreetState {
    fn default() -> DeleteStreetState {
        DeleteStreetState {
            hovered_street: None,
        }
    }
}

impl State for DeleteStreetState {
    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, _: u32, _: &mut Map) {}

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map) {
        if self.hovered_street.is_some() {
            if let Some(street) = map.get_mut(&self.hovered_street.unwrap()) as Option<&mut Street>
            {
                street.set_state(InteractiveElementState::Normal)
            }
        }

        if let Some(hovered_street) = map.get_street_at_position(&mouse_pos, &vec![]) {
            self.hovered_street = Some(hovered_street);

            if let Some(street) = map.get_mut(&self.hovered_street.unwrap()) as Option<&mut Street>
            {
                street.set_state(InteractiveElementState::Hover)
            }
        }
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, _: u32, map: &mut Map) {
        if let Some(hovered_street) = self.hovered_street {
            map.remove_street(&hovered_street);
        }
    }

    fn enter(&self, _map: &mut Map) {}

    fn exit(&self, _map: &mut Map) {}

    fn render(
        &self,
        map: &Map,
        context: &web_sys::CanvasRenderingContext2d,
    ) -> Result<(), wasm_bindgen::JsValue> {
        context.clear_rect(0.0, 0.0, map.width().into(), map.height().into());

        map.render(context)?;

        Ok(())
    }
}
