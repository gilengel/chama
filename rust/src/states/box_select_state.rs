use geo::{Coordinate, Rect};

use crate::{
    interactive_element::{InteractiveElement, InteractiveElementState},
    renderer::PrimitiveRenderer,
    state::State,
    style::Style,
    Map, Renderer,
};

fn default_coordinate() -> Coordinate<f64> {
    Coordinate { x: 0., y: 0. }
}

pub struct BoxSelectState {
    selection_min: Coordinate<f64>,
    selection_max: Coordinate<f64>,
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
            selection_min: default_coordinate(),
            selection_max: default_coordinate(),
            active: false,
        }
    }
}

impl State for BoxSelectState {
    fn mouse_down(&mut self, mouse_pos: Coordinate<f64>, _: u32, _: &mut Map) {
        self.selection_min = mouse_pos;
        self.active = true;
    }

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, _: &mut Map) {
        self.selection_max = mouse_pos;
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, _: u32, map: &mut Map) {
        for intersection in map
            .intersections_within_rectangle_mut(&Rect::new(self.selection_min, self.selection_max))
        {
            intersection.set_state(InteractiveElementState::Selected);
        }

        self.selection_min = default_coordinate();
        self.selection_max = default_coordinate();
        self.active = false;
    }

    fn enter(&mut self, _: &mut Map) {}

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
            Rect::new(self.selection_min, self.selection_max).render(
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
