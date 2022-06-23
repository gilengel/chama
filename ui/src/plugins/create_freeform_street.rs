use std::fmt;

use futures::executor::block_on;
use geo::{simplify::Simplify, Coordinate, LineString, Point};
use plugin_toolbar::toolbar::ToolbarPosition;
use rust_editor::{
    actions::{Action, Redo, Undo},
    gizmo::Id,
    input::{keyboard::Key, mouse},
    plugin::{Plugin, PluginWithOptions},
    renderer::PrimitiveRenderer,
    style::Style,
    ui::app::{EditorError, Shortkey},
};
use rust_macro::editor_plugin;
use uuid::Uuid;
use web_sys::CanvasRenderingContext2d;

use crate::map::{
    map::Map,
    street::{calc_polygon_points, Street},
};

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

    #[option(default = 1., min = 0., max = 10., label = "Simplification Factor")]
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
        if self.street.points().len() == 0 {
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

        editor.plugin_mut(
            move |toolbar_plugin: &mut plugin_toolbar::ToolbarPlugin<Map>| {
                let toolbar = toolbar_plugin
                    .get_or_add_toolbar("primary.edit.modes.street", ToolbarPosition::Left)
                    .unwrap();

                let enabled = Rc::clone(&self.__enabled);

                toolbar
                    .add_toggle_button(
                        "brush",
                        "create_street",
                        "Create Freeform Streets".to_string(),
                        move || *enabled.as_ref().borrow(),
                        move || EditorMessages::ActivatePlugin(CreateFreeformStreet::identifier()),
                    )
                    .unwrap();
            },
        );

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

        app.plugin_mut(move |redo: &mut plugin_undo_redo::Redo<Map>| {
            redo.clear();
        });

        app.plugin_mut(move |undo: &mut plugin_undo_redo::Undo<Map>| {
            undo.push(Rc::clone(&action));
        });

        let cloned_data = app.data().clone();
        app.plugin_mut(move |sync: &mut crate::plugins::sync::Sync| {
            block_on(sync.send(cloned_data.clone()));
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
        context.set_line_width(20.0);
        context.set_stroke_style(&"#2A2A2B".into());

        let line_string = LineString(self.raw_points.clone());

        if line_string.lines().len() == 0 {
            return;
        }

        // TODO better performance: To simplify and calc the polygon each render time is quite costly.
        // a (slightly) better way is to calculate it each time a point is added.
        // We need to find a way to make this really fast
        let line_string = line_string.simplify(&self.simplification_factor);
        let polygon = calc_polygon_points(line_string.lines(), 20.);

        let style = Style {
            border_width: 0,
            border_color: "#0000FF".to_string(),
            background_color: "#FFFFFFF".to_string(),
        };

        polygon.render(&style, context).unwrap();
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
