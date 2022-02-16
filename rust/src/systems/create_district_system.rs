use std::collections::HashMap;

use geo::Coordinate;
use rust_editor::{
    gizmo::Id,
    plugins::{camera::Renderer, plugin::PluginWithOptions},
    system::System,
    InformationLayer,
};
use uuid::Uuid;

use crate::{
    map::{district::create_district_for_street, map::Map},
    Modes,
};

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

impl System<Map, Modes> for CreateDistrictSystem {
    fn mouse_down(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        _: u32,
        _: &mut Map,
        _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) {
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        map: &mut Map,
        _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
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
        _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
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
        _plugins: &HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) -> Result<(), wasm_bindgen::JsValue> {
        map.render(context, additional_information_layer)?;

        Ok(())
    }
}
