use std::collections::HashMap;

use geo::Coordinate;
use rust_editor::{
    gizmo::Id,
    interactive_element::{InteractiveElement, InteractiveElementState},
    plugins::plugin::PluginWithOptions,
    system::System,
    InformationLayer,
};
use uuid::Uuid;

use crate::{
    map::{district::District, map::Map},
    Modes,
};

pub struct DeleteDistrictSystem {
    hovered_district: Option<Uuid>,
}

impl DeleteDistrictSystem {
    pub fn new() -> Self {
        Self {
            hovered_district: None,
        }
    }
}

impl System<Map, Modes> for DeleteDistrictSystem {
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
        if let Some(old_hovered_district) = self.hovered_district {
            let old_hovered_district: &mut District =
                map.district_mut(&old_hovered_district).unwrap();
            old_hovered_district.set_state(InteractiveElementState::Normal);
        }

        if let Some(hovered_district) = map.get_district_at_position(&mouse_pos) {
            let hovered_district: &mut District = map.district_mut(&hovered_district).unwrap();
            hovered_district.set_state(InteractiveElementState::Hover);
            self.hovered_district = Some(hovered_district.id());
        }
    }

    fn mouse_up(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _: u32,
        map: &mut Map,

        _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) {
        if let Some(hovered_district) = map.get_district_at_position(&mouse_pos) {
            map.remove_district(&hovered_district);
            self.hovered_district = None
        }
    }

    fn render(
        &self,
        map: &Map,
        context: &web_sys::CanvasRenderingContext2d,
        _additional_information_layer: &Vec<InformationLayer>,
        _plugins: &HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) -> Result<(), wasm_bindgen::JsValue> {
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
