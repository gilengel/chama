use std::collections::HashSet;

use geo::Coordinate;
use rust_editor::{
    actions::{Action, MultiAction},
    input::{keyboard::Key, mouse},
    interactive_element::{InteractiveElement, InteractiveElementState},
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
    fn clean_hovered_street_state(&self, map: &mut Map) {
        for (_, street) in map.streets_mut() {
            street.set_state(InteractiveElementState::Normal);
        }
    }


}
impl Plugin<Map> for DeleteStreet {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        editor.add_shortkey::<DeleteStreet>(vec![Key::Ctrl, Key::D2])?;

        let toolbar =
            editor.get_or_add_toolbar("primary.edit.modes.street", ToolbarPosition::Left)?;

        let enabled = Rc::clone(&self.__enabled);
        toolbar.add_toggle_button(
            "delete_outline",
            "delete_street",
            "Delete Streets".to_string(),
            move || *enabled.as_ref().borrow(),
            move || EditorMessages::ActivatePlugin(DeleteStreet::identifier()),
        )?;

        Ok(())
    }

    fn shortkey_pressed(&mut self, key: &Shortkey, ctx: &Context<App<Map>>, _: &mut App<Map>) {
        if *key == vec![Key::Ctrl, Key::D2] {
            ctx.link()
                .send_message(EditorMessages::ActivatePlugin(DeleteStreet::identifier()));
        }
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _mouse_movement: Coordinate<f64>,
        _: mouse::Button,
        editor: &mut App<Map>,
    ) -> bool {
        let map = editor.data_mut();
        self.clean_hovered_street_state(map);



        false
    }

    fn mouse_down(
        &mut self,
        mouse_pos: Coordinate<f64>,
        button: mouse::Button,
        editor: &App<Map>,
    ) -> bool {
        if button != mouse::Button::Left {
            return false;
        }

        false
    }

    fn mouse_up(
        &mut self,
        mouse_pos: Coordinate<f64>,
        button: mouse::Button,
        app: &mut App<Map>,
    ) -> bool {
        if button != mouse::Button::Left {
            return false;
        }

        // Only continue if we selected streets before and we are currently hovering a street
        if let (Some(hovered_streets), Some(street_at_mouse_pos)) = (&self.hovered_streets, app.data().get_street_at_position(&mouse_pos, &vec![])) {
            // Don't delete if the currently hovered street was not previously selected
            if !hovered_streets.contains(&street_at_mouse_pos) {
                return false;
            }

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

        false
    }
}

#[cfg(test)]
mod tests {

}
