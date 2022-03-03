use geo::Coordinate;
use rust_editor::{
    interactive_element::{InteractiveElement, InteractiveElementState},
    plugins::plugin::{Plugin, PluginWithOptions}, keys, ui::{app::{EditorError, Shortkey}, toolbar::ToolbarPosition},
};
use rust_macro::editor_plugin;
use uuid::Uuid;

use crate::map::{intersection::Side, map::Map, street::Street};

#[editor_plugin(skip, specific_to=Map, execution=Exclusive)]
pub struct DeleteStreet {
    #[option(skip)]
    hovered_streets: Option<Vec<Uuid>>,
}

impl DeleteStreet {
    fn clean_hovered_street_state(&self, map: &mut Map) {
        for (_, street) in map.streets_mut() {
            street.set_state(InteractiveElementState::Normal);
        }
    }

    fn connected_streets(&self, start: Uuid, map: &Map) -> Vec<Uuid> {
        let mut street = start;
        let mut forward = true;

        let mut streets: Vec<Uuid> = vec![];

        let mut side = Side::Left;

        let mut next = match map.street(&street).unwrap().get_next(side) {
            Some(id) => Some(id),
            None => None,
        };

        while next.is_some()
            && next.unwrap() != start
            && map
                .intersection(&map.street(&street).unwrap().end)
                .unwrap()
                .get_connected_streets()
                .len()
                == 2
        {
            streets.push(street);

            {
                let street = map.street(&street).unwrap();

                if forward {
                    next = match street.get_next(side) {
                        Some(id) => Some(id),
                        None => None,
                    };
                } else {
                    next = match street.get_previous(side) {
                        Some(id) => Some(id),
                        None => None,
                    };
                }

                if next.is_some()
                    && ((street.start == map.street(&next.unwrap()).unwrap().start)
                        || (street.end == map.street(&next.unwrap()).unwrap().end))
                {
                    forward = !forward;

                    side = match side {
                        Side::Left => Side::Right,
                        Side::Right => Side::Left,
                    }
                }
            }

            if let Some(next) = next {
                street = next;
            }
        }

        streets.push(street);

        streets
    }
}
impl Plugin<Map> for DeleteStreet {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        editor.add_shortkey::<DeleteStreet>(keys!["Control", "s"])?;

        let toolbar = editor.get_or_add_toolbar("primary.edit.modes.street", ToolbarPosition::Left)?; 

        let enabled = Rc::clone(&self.__enabled);
        toolbar.add_toggle_button(
            "delete_outline",
            "mumu",
            "Delete Streets".to_string(),
            move || { *enabled.as_ref().borrow() },
            move || { EditorMessages::ActivatePlugin(DeleteStreet::identifier()) },
        )?;


        Ok(())
    }

    fn shortkey_pressed(&mut self, key: &Shortkey, ctx: &Context<App<Map>>) {
        if *key == keys!["Control", "s"] {
            ctx.link().send_message(EditorMessages::ActivatePlugin(DeleteStreet::identifier()));
        }
    }

    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, _button: u32, _map: &mut Map) {}

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _mouse_movement: Coordinate<f64>,
        map: &mut Map,
    ) {
        self.clean_hovered_street_state(map);

        if let Some(hovered_street) = map.get_street_at_position(&mouse_pos, &vec![]) {
            self.hovered_streets = Some(self.connected_streets(hovered_street, map));

            for street in self.hovered_streets.as_ref().unwrap() {
                if let Some(street) = map.street_mut(&street) as Option<&mut Street> {
                    street.set_state(InteractiveElementState::Hover)
                }
            }
        }
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, _button: u32, map: &mut Map) {
        if let Some(hovered_streets) = &self.hovered_streets {
            for street in hovered_streets {
                map.remove_street(&street);
            }
        }
    }
}
