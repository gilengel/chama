use std::fmt;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_macro::editor_plugin;

use crate::{
    map::{district::create_district_for_street, intersection::Side},
    Map,
};
use geo::Coordinate;
use rust_editor::{
    actions::{Action, Redo, Undo},
    gizmo::Id,
    plugins::plugin::{Plugin, PluginWithOptions},
    ui::{
        app::{EditorError, Shortkey},
        toolbar::ToolbarPosition,
    }, input::{keyboard::Key, mouse},
};
use uuid::Uuid;

#[editor_plugin(specific_to=Map, execution=Exclusive)]
pub struct CreateDistrict {
    #[option(skip)]
    hovered_street: Option<Uuid>,

    #[option(
        default = 500.,
        min = 0.,
        max = 100000.,
        label = "Minimum House Side",
        description = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."
    )]
    minimum_house_side: f64,

    #[option(skip)]
    seed: <ChaCha8Rng as SeedableRng>::Seed,
}

struct CreateDistrictAction {
    street: Uuid,
    side: Side,
    minimum_house_side: f64,
    seed: <ChaCha8Rng as SeedableRng>::Seed,

    district: Option<Uuid>,
}

impl Redo<Map> for CreateDistrictAction {
    fn redo(&mut self, map: &mut Map) {
        if let Some(district) = create_district_for_street(
            self.side,
            self.street,
            map,
            self.minimum_house_side,
            self.seed,
        ) {
            let id = district.id().clone();
            map.add_district(district);

            self.district = Some(id);
        }
    }
}

impl Undo<Map> for CreateDistrictAction {
    fn undo(&mut self, map: &mut Map) {
        if let Some(district) = self.district {
            map.remove_district(&district);
        }
    }
}

impl Action<Map> for CreateDistrictAction {}

impl fmt::Display for CreateDistrictAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[create_district] street={}", self.street)
    }
}

impl CreateDistrictAction {
    pub fn new(
        street: Uuid,
        side: Side,
        minimum_house_side: f64,
        seed: <ChaCha8Rng as SeedableRng>::Seed,
    ) -> Self {
        CreateDistrictAction {
            street,
            side,
            minimum_house_side,
            seed,
            district: None,
        }
    }
}

impl Plugin<Map> for CreateDistrict {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        editor.add_shortkey::<CreateDistrict>(vec![Key::Ctrl, Key::D])?;

        let toolbar =
            editor.get_or_add_toolbar("primary.edit.modes.district", ToolbarPosition::Left)?;

        let enabled = Rc::clone(&self.__enabled);
        toolbar.add_toggle_button(
            "maps_home_work",
            "create_district",
            "Create District".to_string(),
            move || *enabled.as_ref().borrow(),
            move || EditorMessages::ActivatePlugin(CreateDistrict::identifier()),
        )?;

        Ok(())
    }

    fn property_updated(&mut self, _: &str, editor: &mut App<Map>) {
        editor
            .data_mut()
            .districts_mut()
            .iter_mut()
            .for_each(|(_, x)| {
                x.minimum_house_side = self.minimum_house_side.clamp(20.0, 1000.0);
                x.update_houses();
            });
    }

    fn shortkey_pressed(&mut self, key: &Shortkey, ctx: &Context<App<Map>>, _: &mut App<Map>) {
        if *key == vec![Key::Ctrl, Key::D] {
            ctx.link()
                .send_message(EditorMessages::ActivatePlugin(CreateDistrict::identifier()));
        }
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _: Coordinate<f64>,
        _: mouse::Button,
        editor: &mut App<Map>,
    ) -> bool {
        match editor.data().get_nearest_street_to_position(&mouse_pos) {
            Some(street) => self.hovered_street = Some(street.id()),
            None => self.hovered_street = None,
        }

        false
    }

    fn mouse_up(&mut self, mouse_pos: Coordinate<f64>, button: mouse::Button, app: &mut App<Map>) -> bool {
        if button != mouse::Button::Left {
            return false;
        }

        if let Some(hovered_street_id) = self.hovered_street {
            let mut create_district_action = CreateDistrictAction::new(
                hovered_street_id,
                app.data()
                    .street(&hovered_street_id)
                    .unwrap()
                    .get_side_of_position(&mouse_pos),
                self.minimum_house_side,
                self.seed,
            );
            create_district_action.execute(app.data_mut());

            let action = Rc::new(RefCell::new(create_district_action));

            app.plugin_mut(move |redo: &mut rust_editor::plugins::redo::Redo<Map>| {
                redo.clear();
            });

            app.plugin_mut(move |undo: &mut rust_editor::plugins::undo::Undo<Map>| {
                undo.push(Rc::clone(&action));
            });
        }

        return false;
    }
}

#[cfg(test)]
mod tests {
    use rust_editor::input::keyboard::Key;
    use rust_editor::plugins::plugin::Plugin;
    use rust_editor::ui::app::App;
    use rust_editor::ui::toolbar::ToolbarPosition;

    use crate::map::map::Map;
    use crate::plugins::create_district::CreateDistrict;

    #[test]
    fn integration_startup_adds_shortcut() {
        let mut app = App::<Map>::default();

        let mut plugin = CreateDistrict::default();
        plugin.startup(&mut app).unwrap();

        assert!(app.has_shortkey(vec![Key::Ctrl, Key::D]))
    }

    #[test]

    fn integration_startup_adds_toolbar_button() {
        let mut app = App::<Map>::default();

        let mut plugin = CreateDistrict::default();
        plugin.startup(&mut app).unwrap();

        let toolbar = app
            .get_or_add_toolbar("primary.edit.modes.district", ToolbarPosition::Left)
            .unwrap();

        assert!(toolbar.has_button("create_district"));
    }
}
