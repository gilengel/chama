use std::collections::HashMap;

use geo::{Coordinate, Rect};
use rust_editor::{
    interactive_element::{InteractiveElement, InteractiveElementState},
    plugins::plugin::PluginWithOptions,
    renderer::PrimitiveRenderer,
    style::Style,
    system::System,
    InformationLayer,
};

use crate::{map::map::Map, Modes};

fn default_coordinate() -> Coordinate<f64> {
    Coordinate { x: 0., y: 0. }
}

pub struct BoxSelectSystem {
    selection_min: Coordinate<f64>,
    selection_max: Coordinate<f64>,
    active: bool,
}

impl Default for BoxSelectSystem {
    fn default() -> Self {
        BoxSelectSystem {
            selection_min: default_coordinate(),
            selection_max: default_coordinate(),
            active: false,
        }
    }
}

impl System<Map, Modes> for BoxSelectSystem {
    fn mouse_down(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _: u32,
        _: &mut Map,
        _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) {
        self.selection_min = mouse_pos;
        self.active = true;
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _: &mut Map,
        _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) {
        self.selection_max = mouse_pos;
    }

    fn mouse_up(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        _: u32,
        map: &mut Map,
        _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) {
        for intersection in map
            .intersections_within_rectangle_mut(&Rect::new(self.selection_min, self.selection_max))
        {
            intersection.set_state(InteractiveElementState::Selected);
        }

        self.selection_min = default_coordinate();
        self.selection_max = default_coordinate();
        self.active = false;
    }

    fn render(
        &self,
        _map: &Map,
        context: &web_sys::CanvasRenderingContext2d,
        _additional_information_layer: &Vec<InformationLayer>,
        _plugins: &HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) -> Result<(), wasm_bindgen::JsValue> {
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
