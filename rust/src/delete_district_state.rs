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
    fn mouse_down(&mut self, _: u32, _: u32, _: u32, _: &mut Map) {

    }

    fn mouse_move(&mut self, x: u32, y: u32, map: &mut Map) {
        let position = Coordinate {
            x: x.into(),
            y: y.into(),
        };
        
        if let Some(hovered_district) = map.get_district_at_position(&position) {
            self.hovered_district = Some(Rc::clone(&hovered_district));
        }
    }

    fn mouse_up(&mut self, x: u32, y: u32, _: u32, map: &mut Map) {
        let position = Coordinate {
            x: x.into(),
            y: y.into(),
        };
        
        if let Some(hovered_district) = map.get_district_at_position(&position) {
            map.remove_district(Rc::clone(&hovered_district));
            self.hovered_district = None
        }
    }

    fn update(&mut self) {

    }

    fn enter(&self) {

    }

    fn exit(&self) {

    }

    fn render(&self, map: &Map, context: &web_sys::CanvasRenderingContext2d) -> Result<(), wasm_bindgen::JsValue> {
        context.clear_rect(0.0, 0.0, map.width().into(), map.height().into());

        map.render(context)?;

        if let Some(hovered_district) = &self.hovered_district {
            let hovered_district = hovered_district.borrow();
            //hovered_street.set_fillstyle("#FF0000");
            hovered_district.render(context)?;
        }

        Ok(())
    }
}