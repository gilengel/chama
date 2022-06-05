use std::fmt;

use geo::{simplify::Simplify, Coordinate, LineString, Point};
use rust_editor::{
    actions::{Action, Redo, Undo},
    gizmo::Id,
    input::{keyboard::Key, mouse},
    plugins::plugin::{Plugin, PluginWithOptions},
    renderer::PrimitiveRenderer,
    style::Style,
    ui::{
        app::{EditorError, Shortkey},
        toolbar::ToolbarPosition,
    }, log,
};
use rust_macro::editor_plugin;
use uuid::Uuid;
use web_sys::CanvasRenderingContext2d;

use crate::map::{map::Map, street::Street};

#[editor_plugin(specific_to=Map, execution=Exclusive)]
pub struct CreateFreeformStreet {
    #[option(skip)]
    raw_points: Vec<Coordinate<f64>>,

    #[option(skip)]
    points: Vec<Point<f64>>,

    #[option(skip)]
    raw_point_style: Style,

    #[option(skip)]
    brush_active: bool,

    #[option(
        default = 1.,
        min = 0.,
        max = 10.,
        label = "Simplification Factor",
        description = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."
    )]
    simplification_factor: f64,
}

pub struct CreateFreeFormStreetAction {
    id: Option<Uuid>,
    street: LineString<f64>,
    
}

impl CreateFreeFormStreetAction {
    pub fn new(street: LineString<f64>) -> Self {
        CreateFreeFormStreetAction { id: None, street }
    }
}

impl Undo<Map> for CreateFreeFormStreetAction {
    fn undo(&mut self, map: &mut Map) {
        if let None = self.id {
            return;
        }
        
        let copy = map.street(&self.id.unwrap()).unwrap().clone();
        map.remove_street(&copy);
    }
}

impl Redo<Map> for CreateFreeFormStreetAction {
    fn redo(&mut self, map: &mut Map) {
        // TODO: Rework editor logic, if I press a button the action is still triggered if the plugin was activated before (e.g. by drawing a street)
        if self.street.points().len() == 0{
            return;
        }

        let street = Street::new(self.street.clone());

        self.id = Some(street.id());
        map.add_street(&street);
    }
}

impl Action<Map> for CreateFreeFormStreetAction {}

impl fmt::Display for CreateFreeFormStreetAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[create_freeform_street]\n\u{251C}",)
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

    fn mouse_down(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        button: mouse::Button,
        _: &App<Map>,
    ) -> bool {
        if button == mouse::Button::Left {
            self.brush_active = true;
        }

        false
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _mouse_movement: Coordinate<f64>,
        _: mouse::Button,
        _: &mut App<Map>,
    ) -> bool {
        if self.brush_active {
            self.raw_points.push(mouse_pos);
        }

        false
    }

    fn mouse_up(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        button: mouse::Button,
        app: &mut App<Map>,
    ) -> bool {
        // Only proceed if the left button was released
        if button != mouse::Button::Left {
            return false;
        }

        self.brush_active = false;

        let line_string = LineString(self.raw_points.clone());
        let simplified = line_string.simplify(&self.simplification_factor);

        let action = Rc::new(RefCell::new(CreateFreeFormStreetAction::new(simplified)));

        action.borrow_mut().execute(app.data_mut());

        app.plugin_mut(move |redo: &mut rust_editor::plugins::redo::Redo<Map>| {
            redo.clear();
        });

        app.plugin_mut(move |undo: &mut rust_editor::plugins::undo::Undo<Map>| {
            undo.push(Rc::clone(&action));
        });

        self.raw_points.clear();

        false
    }

    fn shortkey_pressed(&mut self, key: &Shortkey, ctx: &Context<App<Map>>, _: &mut App<Map>) {
        if *key == vec![Key::Ctrl, Key::A] {
            ctx.link().send_message(EditorMessages::ActivatePlugin(
                CreateFreeformStreet::identifier(),
            ));
        }
    }

    fn render(&self, context: &CanvasRenderingContext2d, _: &App<Map>) {
        context.set_line_width(1.0);
        context.set_stroke_style(&"#2A2A2B".into());

        let line_string = LineString(self.raw_points.clone());
        let style = Style::default();
        line_string.lines().for_each(|line| {
            line.render(&style, context).unwrap();
        });

        /*
        if self.brush_active && !self.raw_points.is_empty() {
            context.begin_path();
            context.move_to(self.raw_points[0].x, self.raw_points[0].y);

            for point in self.raw_points.iter().skip(1) {
                context.line_to(point.x, point.y);
                context.stroke();
                context.move_to(point.x, point.y);
            }

            context.close_path();
            //apply_style(&self.raw_point_style, &context);
        }
        */
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
