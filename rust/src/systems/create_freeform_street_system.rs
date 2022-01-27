

use geo::{simplify::Simplify, Coordinate, LineString};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{
    actions::{
        action::{Action, MultiAction},
        redo::Redo,
        undo::Undo,
    },
    log,
    map::map::InformationLayer,
    renderer::apply_style,
    state::System,
    style::Style,
    Camera, Map,
};

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
    action_stack: MultiAction,
}

impl CreateFreeFormStreetAction {
    pub fn new(raw_points: Vec<Coordinate<f64>>) -> Self {
        CreateFreeFormStreetAction {
            raw_points,
            action_stack: MultiAction::new(),
        }
    }
}

impl Undo for CreateFreeFormStreetAction {
    fn undo(&mut self, map: &mut Map) {
        self.action_stack.undo(map);
    }
}

impl Redo for CreateFreeFormStreetAction {
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

impl Action for CreateFreeFormStreetAction {}

impl CreateFreeFormStreetSystem {
    pub fn new() -> CreateFreeFormStreetSystem {
        CreateFreeFormStreetSystem::default()
    }
}

impl System for CreateFreeFormStreetSystem {
    fn mouse_down(
        &mut self,
        _: Coordinate<f64>,
        button: u32,
        _: &mut Map,
        _actions: &mut Vec<Box<dyn Action>>,
    ) {
        if button == 0 {
            self.brush_active = true;
        }

        log!("{}", button);
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _: &mut Map,
        _actions: &mut Vec<Box<dyn Action>>,
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
        actions: &mut Vec<Box<dyn Action>>,
    ) {
        if button == 0 {
            self.brush_active = false;
        }

        let line_string = LineString(self.raw_points.clone());
        let points = line_string.simplify(&4.0).into_points();

        let mut action = CreateFreeFormStreetAction::new(
            points
                .iter()
                .map(|x| Coordinate { x: x.x(), y: x.y() })
                .collect(),
        );
        action.execute(map);
        actions.push(Box::new(action));

        self.raw_points.clear();
    }

    fn render(
        &self,
        _map: &Map,
        context: &CanvasRenderingContext2d,
        _additional_information_layer: &Vec<InformationLayer>,
        camera: &Camera,
    ) -> Result<(), JsValue> {
        if self.brush_active && !self.raw_points.is_empty() {
            context.begin_path();
            context.move_to(
                self.raw_points[0].x + camera.x as f64,
                self.raw_points[0].y + camera.y as f64,
            );

            for point in self.raw_points.iter().skip(1) {
                context.line_to(point.x + camera.x as f64, point.y + camera.y as f64);
                context.stroke();
                context.move_to(point.x + camera.x as f64, point.y + camera.y as f64)
            }
            //context.close_path();
            apply_style(&self.raw_point_style, &context);
        }

        Ok(())
    }

    fn enter(&mut self, _: &mut Map) {}

    fn exit(&self, _: &mut Map) {}
}
