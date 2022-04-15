use std::fmt;

use geo::{simplify::Simplify, Coordinate, LineString};
use rust_editor::{
    actions::{Action, MultiAction, Redo, Undo},
    plugins::plugin::{Plugin, PluginWithOptions},
    renderer::apply_style,
    style::Style,
    ui::{
        app::{EditorError, Shortkey},
        toolbar::ToolbarPosition,
    }, input::keyboard::Key,
};
use rust_macro::editor_plugin;
use uuid::Uuid;
use web_sys::CanvasRenderingContext2d;

use crate::map::{actions::street::create::CreateStreet, map::Map};

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
        description = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."
    )]
    simplification_factor: f64,
}

pub struct CreateFreeFormStreetAction {
    raw_points: Vec<Coordinate<f64>>,
    action_stack: MultiAction<Map>,
    street_ids: Vec<Uuid>,
}

impl CreateFreeFormStreetAction {
    pub fn new(raw_points: Vec<Coordinate<f64>>) -> Self {
        let street_ids: Vec<Uuid> = raw_points.iter().skip(1).map(|_| Uuid::new_v4()).collect();
        CreateFreeFormStreetAction {
            raw_points,
            action_stack: MultiAction::new(),
            street_ids,
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
        self.action_stack.actions.clear();

        if self.raw_points.is_empty() {
            return;
        }

        /*
            // skip the first n points if at that position already a street exists
            let mut index_to_be_skipped = 0;
            for (index, point) in self.raw_points.iter().enumerate() {
                if map.get_street_at_position(point, &vec![]).is_none() && index != 0 {
                    index_to_be_skipped = index - 1;
                    break;
                }
            }
        */
        let mut previous = &self.raw_points[0];
        for (point, street_id) in self.raw_points.iter().skip(1).zip(self.street_ids.iter()) {
            self.action_stack
                .push(CreateStreet::new(*previous, *point, *street_id));

            previous = point;
        }

        self.action_stack.redo(map);
    }
}

impl Action<Map> for CreateFreeFormStreetAction {}

impl fmt::Display for CreateFreeFormStreetAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[create_freeform_street]\n\u{251C}  {}",
            self.action_stack
        )
    }
}

impl Plugin<Map> for CreateFreeformStreet {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        editor.add_shortkey::<CreateFreeformStreet>(vec![Key::Ctrl, Key::A])?;

        let toolbar =
            editor.get_or_add_toolbar("primary.edit.modes.street", ToolbarPosition::Left)?;

        let enabled = Rc::clone(&self.__enabled);

        toolbar.add_toggle_button(
            "brush",
            "create_street",
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

    fn shortkey_pressed(&mut self, key: &Shortkey, ctx: &Context<App<Map>>, _: &mut App<Map>) {
        if *key == vec![Key::Ctrl, Key::A] {
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

        self.brush_active = false;

        let line_string = LineString(self.raw_points.clone());
        let points = line_string
            .simplify(&self.simplification_factor)
            .into_points();

        let action = Rc::new(RefCell::new(CreateFreeFormStreetAction::new(
            points
                .iter()
                .map(|x| Coordinate { x: x.x(), y: x.y() })
                .collect(),
        )));

        action.borrow_mut().execute(app.data_mut());

        app.plugin_mut(move |redo: &mut rust_editor::plugins::redo::Redo<Map>| {
            redo.clear();
        });

        app.plugin_mut(move |undo: &mut rust_editor::plugins::undo::Undo<Map>| {
            undo.push(Rc::clone(&action));
        });

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

#[cfg(test)]
mod tests {
    use rust_editor::input::keyboard::Key;
    use rust_editor::plugins::plugin::Plugin;
    use rust_editor::ui::app::App;
    use rust_editor::ui::toolbar::ToolbarPosition;

    use crate::map::map::Map;
    use crate::plugins::create_freeform_street::CreateFreeformStreet;

    #[test]
    fn integration_startup_adds_shortcut() {
        let mut app = App::<Map>::default();

        let mut plugin = CreateFreeformStreet::default();
        plugin.startup(&mut app).unwrap();

        assert!(app.has_shortkey(vec![Key::Ctrl, Key::A]))
    }

    #[test]

    fn integration_startup_adds_toolbar_button() {
        let mut app = App::<Map>::default();

        let mut plugin = CreateFreeformStreet::default();
        plugin.startup(&mut app).unwrap();

        let toolbar = app
            .get_or_add_toolbar("primary.edit.modes.street", ToolbarPosition::Left)
            .unwrap();

        assert!(toolbar.has_button("create_street"));
    }
}
