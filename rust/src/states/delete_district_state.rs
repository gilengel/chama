

use geo::Coordinate;
use uuid::Uuid;

use crate::{
    interactive_element::{InteractiveElementState, InteractiveElement},

    state::State,
    Renderer, Camera, map::{district::District, map::{Map, InformationLayer}}, gizmo::Id,
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
            let old_hovered_district: &mut District = map.district_mut(&old_hovered_district).unwrap();
            old_hovered_district.set_state(InteractiveElementState::Normal);
        }

        if let Some(hovered_district) = map.get_district_at_position(&mouse_pos) {
            let hovered_district: &mut District = map.district_mut(&hovered_district).unwrap();
            hovered_district.set_state(InteractiveElementState::Hover);
            self.hovered_district = Some(hovered_district.id());
        }
    }

    fn mouse_up(&mut self, mouse_pos: Coordinate<f64>, _: u32, map: &mut Map) {
        if let Some(hovered_district) = map.get_district_at_position(&mouse_pos) {
            map.remove_district(&hovered_district);
            self.hovered_district = None
        }
    }

    fn enter(&mut self, _map: &mut Map) {}

    fn exit(&self, _map: &mut Map) {}

    fn render(
        &self,
        map: &Map,
        context: &web_sys::CanvasRenderingContext2d, additional_information_layer: &Vec<InformationLayer>, camera: &Camera
    ) -> Result<(), wasm_bindgen::JsValue> {
        map.render(&context, additional_information_layer, camera)?;

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
