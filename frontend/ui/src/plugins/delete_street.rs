use geo::Coordinate;
use rust_editor::{
    interactive_element::{InteractiveElement, InteractiveElementState},
    keys,
    plugins::plugin::{Plugin, PluginWithOptions},
    ui::{
        app::{EditorError, Shortkey},
        toolbar::ToolbarPosition,
    },
};
use rust_macro::editor_plugin;
use uuid::Uuid;

use crate::map::intersection::Intersection;
use crate::map::{intersection::Side, map::Map, street::Street};

#[editor_plugin(skip, specific_to=Map, execution=Exclusive)]
pub struct DeleteStreet {
    #[option(skip)]
    hovered_streets: Option<Vec<Uuid>>,
}

impl DeleteStreet {
    fn iter<'a, FnNextIntersection, FnNextStreet>(
        &self,
        start: Uuid,
        map: &'a Map,
        fn_next_intersection: FnNextIntersection,
        fn_next_street: FnNextStreet,
    ) -> Vec<Uuid>
    where
        FnNextIntersection: Fn(&Uuid) -> Option<&'a Intersection>,
        FnNextStreet: Fn(bool, &Street, Side) -> Option<Uuid>,
    {
        let mut street = start;
        let mut forward = true;

        let mut streets: Vec<Uuid> = vec![street];

        let mut side = Side::Left;
        let mut next = fn_next_street(forward, map.street(&street).unwrap(), side);

        while next.is_some()
            && next.unwrap() != start
            && fn_next_intersection(&street)
                .unwrap()
                .get_connected_streets()
                .len()
                <= 2
        {
            streets.push(street);

            {
                let street = map.street(&street).unwrap();

                next = fn_next_street(forward, street, side);

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

    fn iter_backward(&self, start: Uuid, map: &Map) -> Vec<Uuid> {
        self.iter(
            start,
            map,
            |uuid| map.intersection(&map.street(&uuid).unwrap().start),
            |forward, street, side| {
                if forward {
                    street.get_previous(side)
                } else {
                    street.get_next(side)
                }
            },
        )
    }

    fn iter_forward(&self, start: Uuid, map: &Map) -> Vec<Uuid> {
        self.iter(
            start,
            map,
            |uuid| map.intersection(&map.street(&uuid).unwrap().end),
            |forward, street, side| {
                if forward {
                    street.get_next(side)
                } else {
                    street.get_previous(side)
                }
            },
        )
    }

    fn clean_hovered_street_state(&self, map: &mut Map) {
        for (_, street) in map.streets_mut() {
            street.set_state(InteractiveElementState::Normal);
        }
    }

    fn connected_streets(&self, start: Uuid, map: &Map) -> Vec<Uuid> {
        let mut streets = self.iter_backward(start, map);
        streets.append(&mut self.iter_forward(start, map));

        streets
    }
}
impl Plugin<Map> for DeleteStreet {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        editor.add_shortkey::<DeleteStreet>(keys!["Control", "2"])?;

        let toolbar =
            editor.get_or_add_toolbar("primary.edit.modes.street", ToolbarPosition::Left)?;

        let enabled = Rc::clone(&self.__enabled);
        toolbar.add_toggle_button(
            "delete_outline",
            "mumu",
            "Delete Streets".to_string(),
            move || *enabled.as_ref().borrow(),
            move || EditorMessages::ActivatePlugin(DeleteStreet::identifier()),
        )?;

        Ok(())
    }

    fn shortkey_pressed(&mut self, key: &Shortkey, ctx: &Context<App<Map>>, _: &mut App<Map>) {
        if *key == keys!["Control", "2"] {
            ctx.link()
                .send_message(EditorMessages::ActivatePlugin(DeleteStreet::identifier()));
        }
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _mouse_movement: Coordinate<f64>,
        editor: &mut App<Map>,
    ) {
        let map = editor.data_mut();
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

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, _button: u32, app: &mut App<Map>) {
        if let Some(hovered_streets) = &self.hovered_streets {
            for street in hovered_streets {
                app.data_mut().remove_street(&street);
            }
        }
    }
}
