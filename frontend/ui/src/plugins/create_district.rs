use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_macro::editor_plugin;

use crate::{map::district::create_district_for_street, Map};
use geo::Coordinate;
use rust_editor::{
    gizmo::Id,
    keys,
    plugins::plugin::{Plugin, PluginWithOptions},
    ui::{
        app::{EditorError, Shortkey},
        toolbar::ToolbarPosition,
    }, log,
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
        description = "Muu"
    )]
    minimum_house_side: f64,

    #[option(skip)]
    seed: <ChaCha8Rng as SeedableRng>::Seed
}

impl Plugin<Map> for CreateDistrict {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        log!("{:?}", self.seed);
        editor.add_shortkey::<CreateDistrict>(keys!["Control", "d"])?;

        let toolbar =
            editor.get_or_add_toolbar("primary.edit.modes.district", ToolbarPosition::Left)?;

        let enabled = Rc::clone(&self.__enabled);
        toolbar.add_toggle_button(
            "maps_home_work",
            "mumumu",
            "Create District".to_string(),
            move || *enabled.as_ref().borrow(),
            move || EditorMessages::ActivatePlugin(CreateDistrict::identifier()),
        )?;

        Ok(())
    }

    fn property_updated(&mut self, property: &str, editor: &mut App<Map>) {
        editor.data_mut().districts_mut().iter_mut().for_each(|(_, x)| {
            x.minimum_house_side = self.minimum_house_side.clamp(20.0, 1000.0);
            x.update_houses();
        });
    }

    fn shortkey_pressed(&mut self, key: &Shortkey, ctx: &Context<App<Map>>, _: &mut App<Map>) {
        if *key == keys!["Control", "d"] {
            ctx.link()
                .send_message(EditorMessages::ActivatePlugin(CreateDistrict::identifier()));
        }
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _mouse_movement: Coordinate<f64>,
        editor: &mut App<Map>,
    ) {
        match editor.data().get_nearest_street_to_position(&mouse_pos) {
            Some(street) => self.hovered_street = Some(street.id()),
            None => self.hovered_street = None,
        }
    }

    fn mouse_up(&mut self, mouse_pos: Coordinate<f64>, button: u32, app: &mut App<Map>) {
        if button != 0 {
            return;
        }

        if let Some(hovered_street_id) = self.hovered_street {
            let hovered_street = app.data().street(&hovered_street_id).unwrap();
            let side = hovered_street.get_side_of_position(&mouse_pos);

            if let Some(district) =
                create_district_for_street(side, hovered_street_id, app.data_mut(), self.minimum_house_side, self.seed)
            {
                app.data_mut().add_district(district);
            }
        }
    }
}
