use geo::Coordinate;
use uuid::Uuid;

use crate::{
    district::create_district_for_street,
    map::{Get, InformationLayer, Map},
    state::State,
    street::Street,
    Renderer,
};

pub struct CreateDistrictState {
    hovered_street: Option<Uuid>,
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

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map) {
        match map.get_nearest_street_to_position(&mouse_pos) {
            Some(street) => self.hovered_street = Some(street.id),
            None => self.hovered_street = None,
        }
    }

    fn mouse_up(&mut self, mouse_pos: Coordinate<f64>, _: u32, map: &mut Map) {
        if let Some(hovered_street_id) = self.hovered_street {
            let hovered_street = map.get(&hovered_street_id).unwrap() as &Street;
            let side = hovered_street.get_side_of_position(&mouse_pos);

            if let Some(district) = create_district_for_street(side, hovered_street_id, map) {
                map.add_district(district);
            }

            
        }
    }

    fn enter(&self, _map: &mut Map) {}

    fn exit(&self, _map: &mut Map) {}

    fn render(
        &self,
        map: &Map,
        context: &web_sys::CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
    ) -> Result<(), wasm_bindgen::JsValue> {
        context.clear_rect(0.0, 0.0, map.width().into(), map.height().into());

        map.render(context, additional_information_layer)?;

        Ok(())
    }
}
