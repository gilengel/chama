use std::{rc::Rc, cell::RefCell};

use geo::Coordinate;

use crate::{state::State, map::Map, Renderer, street::Street};

pub struct DeleteStreetState {
    hovered_street: Option<Rc<RefCell<Street>>>
}

impl DeleteStreetState {
    pub fn new() -> Self {
        DeleteStreetState { 
            hovered_street: None,
        }
    }
}
impl State for DeleteStreetState {
    fn mouse_down(&mut self, _: u32, _: u32, _: u32, _: &mut Map) {

    }

    fn mouse_move(&mut self, x: u32, y: u32, map: &mut Map) {
        let position = Coordinate {
            x: x.into(),
            y: y.into(),
        };
        
        if let Some(hovered_street ) = map.get_street_at_position(&position) {
            self.hovered_street = Some(Rc::clone(&hovered_street));
        }
    }

    fn mouse_up(&mut self, x: u32, y: u32, _: u32, map: &mut Map) {
        let position = Coordinate {
            x: x.into(),
            y: y.into(),
        };
        
        if let Some(hovered_street ) = map.get_street_at_position(&position) {
            map.remove_street(Rc::clone(&hovered_street));
            self.hovered_street = None
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

        if let Some(hovered_street) = &self.hovered_street {
            let hovered_street = hovered_street.borrow();
            //hovered_street.set_fillstyle("#FF0000");
            hovered_street.render(context)?;
        }

        Ok(())
    }
}