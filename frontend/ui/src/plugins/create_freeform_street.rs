use geo::{simplify::Simplify, Coordinate, LineString};
use rust_editor::{
    actions::{Action, MultiAction, Redo, Undo},
    keys, log,
    plugins::plugin::{Plugin, PluginWithOptions},
    renderer::apply_style,
    style::Style,
    ui::{
        app::{EditorError, Shortkey},
        toolbar::ToolbarPosition,
    },
};
use rust_macro::editor_plugin;
use uuid::Uuid;
use web_sys::CanvasRenderingContext2d;

use crate::map::map::Map;

#[editor_plugin(specific_to=Map, execution=Exclusive)]
pub struct CreateFreeformStreet {
    #[option(skip)]
    raw_points: Vec<Coordinate<f64>>,

    #[option(skip)]
    raw_point_style: Style,

    #[option(skip)]
    brush_active: bool,

    #[option(
        default = 4.,
        min = 0.,
        max = 10.,
        label = "Simplification Factor",
        description = "Muu"
    )]
    simplification_factor: f64,
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
        if self.raw_points.is_empty() {
            return;
        }

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
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        editor.add_shortkey::<CreateFreeformStreet>(keys!["Control", "a"])?;

        let toolbar =
            editor.get_or_add_toolbar("primary.edit.modes.street", ToolbarPosition::Left)?;

        let enabled = Rc::clone(&self.__enabled);

        toolbar.add_toggle_button(
            "brush",
            "mumu",
            "Create Freeform Streets".to_string(),
            move || *enabled.as_ref().borrow(),
            move || EditorMessages::ActivatePlugin(CreateFreeformStreet::identifier()),
        )?;

        Ok(())
    }
    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, button: u32, _: &App<Map>) {
        if button == 0 {
            self.brush_active = true;
        }
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _mouse_movement: Coordinate<f64>,
        _: &mut App<Map>,
    ) {
        if self.brush_active {
            self.raw_points.push(mouse_pos);
        }
    }

    fn shortkey_pressed(&mut self, key: &Shortkey, ctx: &Context<App<Map>>) {
        if *key == keys!["Control", "a"] {
            ctx.link().send_message(EditorMessages::ActivatePlugin(
                CreateFreeformStreet::identifier(),
            ));
        }
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, button: u32, app: &mut App<Map>) {
        // Only proceed if the left button was released
        if button != 0 {
            return;
        }

        log!("{}", self.simplification_factor);

        self.brush_active = false;

        let line_string = LineString(self.raw_points.clone());
        let points = line_string
            .simplify(&self.simplification_factor)
            .into_points();

        let mut action = CreateFreeFormStreetAction::new(
            points
                .iter()
                .map(|x| Coordinate { x: x.x(), y: x.y() })
                .collect(),
        );
        action.execute(app.data_mut());

        /*
        // If an undo plugin is registered store the action into it to make in undoable
        if let Some(undo) = get_plugin_mut::<Map, rust_editor::plugins::undo::Undo<Map>>(plugins) {
            undo.push(Box::new(action));
        }
        */

        self.raw_points.clear();
    }

    fn render(&self, context: &CanvasRenderingContext2d, _: &App<Map>) {
        if self.brush_active && !self.raw_points.is_empty() {
            context.begin_path();
            context.move_to(self.raw_points[0].x, self.raw_points[0].y);

            for point in self.raw_points.iter().skip(1) {
                context.line_to(point.x, point.y);
                context.stroke();
                context.move_to(point.x, point.y);
            }

            context.close_path();
            apply_style(&self.raw_point_style, &context);
        }
    }
}
