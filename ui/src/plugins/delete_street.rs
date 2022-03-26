use std::collections::HashSet;

use geo::Coordinate;
use rust_editor::{
    actions::{Action, MultiAction},
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

use crate::map::actions::street::delete::DeleteStreet as ActionDeleteStreet;

use crate::map::intersection::Intersection;
use crate::map::{intersection::Side, map::Map, street::Street};

#[editor_plugin(skip, specific_to=Map, execution=Exclusive)]
pub struct DeleteStreet {
    #[option(skip)]
    hovered_streets: Option<HashSet<Uuid>>,
}

impl DeleteStreet {
    fn iter<'a, FnNextIntersection, FnNextStreet>(
        &self,
        start: Uuid,
        map: &'a Map,
        fn_next_intersection: FnNextIntersection,
        fn_next_street: FnNextStreet,
    ) -> HashSet<Uuid>
    where
        FnNextIntersection: Fn(&Uuid) -> Option<&'a Intersection>,
        FnNextStreet: Fn(bool, &Street, Side) -> Option<Uuid>,
    {
        let mut street = start;
        let mut forward = true;

        let mut streets = HashSet::new();

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
            streets.insert(street);

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

        //streets.push(street);

        streets
    }

    fn iter_backward(&self, start: Uuid, map: &Map) -> HashSet<Uuid> {
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

    fn iter_forward(&self, start: Uuid, map: &Map) -> HashSet<Uuid> {
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

    fn connected_streets(&self, start: Uuid, map: &Map) -> HashSet<Uuid> {
        let a = self.iter_backward(start, map);
        let b = self.iter_forward(start, map);
        let union: HashSet<_> = a.union(&b).collect();
        union.into_iter().map(|x| *x).collect()
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
            let action = Rc::new(RefCell::new(MultiAction::new()));
            for street in hovered_streets {
                action
                    .as_ref()
                    .borrow_mut()
                    .push(ActionDeleteStreet::new(*street));
            }

            action.as_ref().borrow_mut().execute(app.data_mut());

            app.plugin_mut(move |redo: &mut rust_editor::plugins::redo::Redo<Map>| {
                redo.clear();
            });

            app.plugin_mut(move |undo: &mut rust_editor::plugins::undo::Undo<Map>| {
                undo.push(Rc::clone(&action));
            });
        }
    }
}