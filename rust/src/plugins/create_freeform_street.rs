use geo::{simplify::Simplify, Coordinate, LineString};
use rust_editor::{
    actions::{Action, MultiAction, Redo, Undo},
    plugins::plugin::Plugin,
    renderer::apply_style,
    style::Style,
};
use rust_macro::editor_plugin;
use uuid::Uuid;
use web_sys::CanvasRenderingContext2d;

use crate::{map::map::Map, Modes};

#[editor_plugin(specific_to=Map, execution=Exclusive, shortkey=Ctrl+1)]
pub struct CreateFreeformStreet {
    #[option(skip)]
    raw_points: Vec<Coordinate<f64>>,

    #[option(skip)]
    raw_point_style: Style,

    #[option(skip)]
    brush_active: bool,
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

impl Plugin<Map> for CreateFreeformStreet {
    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, button: u32, map: &mut Map) {
        if button == 0 {
            self.brush_active = true;
        }
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        mouse_movement: Coordinate<f64>,
        _data: &mut Map,
    ) {
        if self.brush_active {
            self.raw_points.push(mouse_pos);
        }
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, button: u32, map: &mut Map) {
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

    fn render(&self, context: &CanvasRenderingContext2d) {
        if self.brush_active && !self.raw_points.is_empty() {
            /*
            let offset = match get_plugin::<Map, Modes, Camera>(plugins) {
                Some(x) => Coordinate { x: x.x(), y: x.y() },
                None => Coordinate { x: 0., y: 0. },
            };
            */
            let offset = Coordinate { x: 0., y: 0. };

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
    }
}
