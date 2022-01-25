use geo::{Coordinate, Rect};

use crate::{renderer::PrimitiveRenderer, state::State, style::Style, Map, Renderer, interactive_element::{InteractiveElement, InteractiveElementState}};

pub struct BoxSelectState {
    selection_rect: Rect<f64>,
    active: bool,
}

impl BoxSelectState {
    pub fn new() -> Self {
        BoxSelectState::default()
    }
}

impl Default for BoxSelectState {
    fn default() -> Self {
        BoxSelectState {
            selection_rect: default_rect(),
            active: false,
        }
    }
}

fn default_rect() -> Rect<f64> {
    Rect::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 0., y: 0. })
}

impl State for BoxSelectState {
    fn mouse_down(&mut self, mouse_pos: Coordinate<f64>, _: u32, _: &mut Map) {
        self.selection_rect.set_min(mouse_pos);
        self.active = true;
    }

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, _: &mut Map) {
        self.selection_rect.set_max(mouse_pos);
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, _: u32, map: &mut Map) {
        for intersection in map.intersections_within_rectangle_mut(&self.selection_rect) {
            intersection.set_state(InteractiveElementState::Selected);
        }

        self.selection_rect = default_rect();
        self.active = false;
    }

    fn enter(&self, _: &mut Map) {}

    fn exit(&self, _: &mut Map) {}

    fn render(
        &self,
        map: &Map,
        context: &web_sys::CanvasRenderingContext2d,
        additional_information_layer: &Vec<crate::map::map::InformationLayer>,
        camera: &crate::Camera,
    ) -> Result<(), wasm_bindgen::JsValue> {
        map.render(context, additional_information_layer, camera)?;

        if self.active {
            self.selection_rect.render(
                &Style {
                    border_width: 2,
                    border_color: "rgba(255, 255, 255, 0.1)".to_string(),
                    background_color: "rgba(255, 255, 255, 0.05)".to_string(),
                },
                context,
            )?;
        }

        Ok(())
    }
}
