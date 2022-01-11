use std::{cell::RefCell, rc::Rc};

use geo::Coordinate;

use crate::{map::Map, state::State, street::Street, Renderer};

pub struct CreateDistrictState {
    hovered_street: Option<Rc<RefCell<Street>>>,
}

impl CreateDistrictState {
    pub fn new() -> Self {
        CreateDistrictState {
            hovered_street: None,
        }
    }
}

impl Default for CreateDistrictState {
    fn default() -> CreateDistrictState {
        CreateDistrictState {
            hovered_street: None,
        }
    }
}

impl State for CreateDistrictState {
    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, _: u32, _: &mut Map) {}

    fn mouse_move(&mut self, _mouse_pos: Coordinate<f64>, _map: &mut Map) {
        /*
        if let Some(hovered_street) = map.get_nearest_street_to_position(&position) {
            self.hovered_street = Some(Rc::clone(&hovered_street));
        } else {
            self.hovered_street = None;
        }
        */
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, _: u32, _map: &mut Map) {
        /*
        if self.hovered_street.is_some() {
            let hovered_street = self.hovered_street.as_ref().unwrap();
            let street = hovered_street.as_ref().borrow();

            let side = street.get_side_of_position(&position);
            if let Some(district) = create_district_for_street(side, Rc::clone(&hovered_street)) {
                map.add_district(Rc::new(RefCell::new(district)));
            }
        }
        */
    }


    fn enter(&self, map: &mut Map) {}

    fn exit(&self, map: &mut Map) {}

    fn render(
        &self,
        map: &Map,
        context: &web_sys::CanvasRenderingContext2d,
    ) -> Result<(), wasm_bindgen::JsValue> {
        context.clear_rect(0.0, 0.0, map.width().into(), map.height().into());

        map.render(context)?;

        if let Some(hovered_street) = &self.hovered_street {
            let _hovered_street = hovered_street.borrow();
            //hovered_street.set_fillstyle("#FF0000");
            //hovered_street.render(context)?;
        }

        Ok(())
    }
}
