use std::{cell::RefCell, rc::Rc};

use geo::Coordinate;
use uuid::Uuid;

use crate::{
    district::District,
    interactive_element::{InteractiveElementState, InteractiveElement},
    map::{GetMut, Map},
    state::State,
    Renderer,
};

pub struct DeleteDistrictState {
    hovered_district: Option<Uuid>,
}

impl DeleteDistrictState {
    pub fn new() -> Self {
        DeleteDistrictState {
            hovered_district: None,
        }
    }
}

impl Default for DeleteDistrictState {
    fn default() -> DeleteDistrictState {
        DeleteDistrictState {
            hovered_district: None,
        }
    }
}

impl State for DeleteDistrictState {
    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, _: u32, _: &mut Map) {}

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map) {
        if let Some(old_hovered_district) = self.hovered_district {
            let old_hovered_district: &mut District = map.get_mut(&old_hovered_district).unwrap();
            old_hovered_district.set_state(InteractiveElementState::Normal);
        }

        if let Some(hovered_district) = map.get_district_at_position(&mouse_pos) {
            let hovered_district: &mut District = map.get_mut(&hovered_district).unwrap();
            hovered_district.set_state(InteractiveElementState::Hover);
            self.hovered_district = Some(hovered_district.id);
        }
    }

    fn mouse_up(&mut self, mouse_pos: Coordinate<f64>, _: u32, map: &mut Map) {
        if let Some(hovered_district) = map.get_district_at_position(&mouse_pos) {
            map.remove_district(&hovered_district);
            self.hovered_district = None
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
}
