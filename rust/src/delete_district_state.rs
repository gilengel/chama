use std::{rc::Rc, cell::RefCell};

use geo::Coordinate;

use crate::{state::State, map::Map, Renderer, district::District};

pub struct DeleteDistrictState {
    hovered_district: Option<Rc<RefCell<District>>>
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
            hovered_district: None
        }
    }
}

impl State for DeleteDistrictState {
    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, _: u32, _: &mut Map) {

    }

    fn mouse_move(&mut self, _mouse_pos: Coordinate<f64>, _map: &mut Map) {
        /*
        if let Some(old_hovered_district) = &self.hovered_district {
            old_hovered_district
                .borrow_mut()
                .set_state(InteractiveElementState::Normal);
        }

        if let Some(hovered_district) = map.get_district_at_position(&position) {
            {
                hovered_district
                    .borrow_mut()
                    .set_state(InteractiveElementState::Hover);
            }
            self.hovered_district = Some(Rc::clone(&hovered_district));
        }
        */
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, _: u32, _map: &mut Map) {
        /*
        if let Some(hovered_district) = map.get_district_at_position(&position) {
            map.remove_district(Rc::clone(&hovered_district));
            self.hovered_district = None
        }
        */
    }

    fn enter(&self, _map: &mut Map) {

    }

    fn exit(&self, _map: &mut Map) {

    }

    fn render(&self, map: &Map, context: &web_sys::CanvasRenderingContext2d) -> Result<(), wasm_bindgen::JsValue> {
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