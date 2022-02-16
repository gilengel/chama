use std::collections::HashMap;

use geo::{simplify::Simplify, Coordinate, LineString};
use rust_editor::{
    actions::{Action, MultiAction, Redo, Undo},
    plugins::{plugin::{PluginWithOptions}, camera::Camera},
    renderer::apply_style,
    style::Style,
    system::System,
    InformationLayer, get_plugin,
};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{map::map::Map, Modes};

pub struct CreateFreeFormStreetSystem {
    raw_points: Vec<Coordinate<f64>>,
    raw_point_style: Style,

    brush_active: bool,
}

impl Default for CreateFreeFormStreetSystem {
    fn default() -> Self {
        CreateFreeFormStreetSystem {
            raw_points: Vec::new(),
            raw_point_style: Style {
                border_width: 15,
                border_color: "#2A2A2B".to_string(),
                background_color: "#2A2A2B".to_string(),
            },
            brush_active: false,
        }
    }
}

struct CreateFreeFormStreetAction {
    raw_points: Vec<Coordinate<f64>>,
    action_stack: MultiAction<Map>,
}

impl CreateFreeFormStreetAction {
    pub fn new(raw_points: Vec<Coordinate<f64>>) -> Self {
        CreateFreeFormStreetAction {
            raw_points,
            action_stack: MultiAction::new(),
        }
    }
}

impl Undo<Map> for CreateFreeFormStreetAction {
    fn undo(&mut self, map: &mut Map) {
        self.action_stack.undo(map);
    }
}

impl Redo<Map> for CreateFreeFormStreetAction {
    fn redo(&mut self, map: &mut Map) {
        let _intersections: Vec<Uuid> = vec![];

        let mut index_to_be_skipped = 0;
        for (index, point) in self.raw_points.iter().enumerate() {
            if map.get_street_at_position(point, &vec![]).is_none() && index != 0 {
                index_to_be_skipped = index - 1;
                break;
            }
        }

        let mut previous = &self.raw_points[index_to_be_skipped];
        for point in self.raw_points.iter().skip(index_to_be_skipped + 1) {
            self.action_stack
                .actions
                .push(map.create_street(&previous, point, 10.0));

            previous = point;
        }
    }
}

impl Action<Map> for CreateFreeFormStreetAction {}

impl CreateFreeFormStreetSystem {
    pub fn new() -> CreateFreeFormStreetSystem {
        CreateFreeFormStreetSystem::default()
    }
}

impl System<Map, Modes> for CreateFreeFormStreetSystem {
    fn mouse_down(
        &mut self,
        _: Coordinate<f64>,
        button: u32,
        _: &mut Map,

       _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>
    ) {
        if button == 0 {
            self.brush_active = true;
        }
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _: &mut Map,

       _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>
    ) {
        if self.brush_active {
            self.raw_points.push(mouse_pos);
        }
    }

    fn mouse_up(
        &mut self,
        _: Coordinate<f64>,
        button: u32,
        map: &mut Map,
       _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>
    ) {
        // Only proceed if the left button was released
        if button != 0 {
            return;
        }

        self.brush_active = false;

        let line_string = LineString(self.raw_points.clone());
        let points = line_string.simplify(&4.0).into_points();

        let mut action = CreateFreeFormStreetAction::new(
            points
                .iter()
                .map(|x| Coordinate { x: x.x(), y: x.y() })
                .collect(),
        );
        action.execute(map);

        /*
        // If an undo plugin is registered store the action into it to make in undoable
        if let Some(undo) = get_plugin_mut::<Map, rust_editor::plugins::undo::Undo<Map>>(plugins) {
            undo.push(Box::new(action));
        }
        */

        self.raw_points.clear();
    }

    fn render(
        &self,
        _map: &Map,
        context: &CanvasRenderingContext2d,
        _additional_information_layer: &Vec<InformationLayer>,
        plugins: &HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>
    ) -> Result<(), JsValue> {
        if self.brush_active && !self.raw_points.is_empty() {
  
            let offset = match get_plugin::<Map, Modes, Camera>(plugins) {
                Some(x) => Coordinate { x: x.x(), y: x.y() },
                None => Coordinate { x: 0., y: 0. },
            };

            context.begin_path();
            context.move_to(
                self.raw_points[0].x + offset.x,
                self.raw_points[0].y + offset.y,
            );

            for point in self.raw_points.iter().skip(1) {
                context.line_to(point.x + offset.x, point.y + offset.y);
                context.stroke();
                context.move_to(point.x + offset.x, point.y + offset.y);
            }

            context.close_path();
            apply_style(&self.raw_point_style, &context);
        }

        Ok(())
    }
}
