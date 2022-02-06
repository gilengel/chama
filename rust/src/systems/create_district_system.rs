use geo::Coordinate;
use rust_editor::{
    gizmo::Id,
    plugins::{camera::Renderer, plugin::Plugin},
    system::System,
    InformationLayer,
};
use uuid::Uuid;

use crate::map::{district::create_district_for_street, map::Map};

pub struct CreateDistrictSystem {
    hovered_street: Option<Uuid>,
}

impl Default for CreateDistrictSystem {
    fn default() -> CreateDistrictSystem {
        CreateDistrictSystem {
            hovered_street: None,
        }
    }
}

impl System<Map> for CreateDistrictSystem {
    fn mouse_down(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        _: u32,
        _: &mut Map,
        _plugins: &mut Vec<Box<dyn Plugin<Map>>>
    ) {
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        map: &mut Map,        
        _plugins: &mut Vec<Box<dyn Plugin<Map>>>
    ) {
        match map.get_nearest_street_to_position(&mouse_pos) {
            Some(street) => self.hovered_street = Some(street.id()),
            None => self.hovered_street = None,
        }
    }

    fn mouse_up(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _: u32,
        map: &mut Map,
        _plugins: &mut Vec<Box<dyn Plugin<Map>>>
    ) {
        if let Some(hovered_street_id) = self.hovered_street {
            let hovered_street = map.street(&hovered_street_id).unwrap();
            let side = hovered_street.get_side_of_position(&mouse_pos);

            if let Some(district) = create_district_for_street(side, hovered_street_id, map) {
                map.add_district(district);
            }
        }
    }

    fn render(
        &self,
        map: &Map,
        context: &web_sys::CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
        _plugins: &Vec<Box<dyn Plugin<Map>>>
        
    ) -> Result<(), wasm_bindgen::JsValue> {
        map.render(context, additional_information_layer)?;

        Ok(())
    }
}
