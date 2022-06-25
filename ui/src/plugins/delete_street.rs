use crate::map::actions::street::delete::DeleteStreet as ActionDeleteStreet;
use geo::Coordinate;
use plugin_toolbar::toolbar::{ToolbarPosition};
use rust_editor::{
    actions::Action,
    input::{keyboard::Key, mouse},
    interactive_element::{InteractiveElement, InteractiveElementState},
    plugin::{Plugin, PluginWithOptions},
    ui::app::{EditorError, Shortkey},
};
use rust_macro::editor_plugin;

use crate::map::map::Map;

#[editor_plugin(skip, specific_to=Map, execution=Exclusive)]
pub struct DeleteStreet {}

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

        editor.plugin_mut(
            move |toolbar_plugin: &mut plugin_toolbar::ToolbarPlugin<Map>| {
                let toolbar = toolbar_plugin
                    .get_or_add_toolbar("primary.edit.modes.street", ToolbarPosition::Left)
                    .unwrap();

                let enabled = Rc::clone(&self.__enabled);
                toolbar
                    .add_toggle_button(
                        "delete_outline",
                        "delete_street",
                        "Delete Streets".to_string(),
                        move || *enabled.as_ref().borrow(),
                        move || EditorMessages::ActivatePlugin(DeleteStreet::identifier()),
                    )
                    .unwrap();
            },
        );

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

        for (_, street) in map.streets_mut() {
            if street.is_point_on_street(&mouse_pos) {
                street.set_state(InteractiveElementState::Hover);
            }
        }

        false
    }

    fn mouse_down(&mut self, _: Coordinate<f64>, button: mouse::Button, _: &App<Map>) -> bool {
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

        if let Some(street) = app.data().get_street_at_position(&mouse_pos, &vec![]) {
            let action = Rc::new(RefCell::new(ActionDeleteStreet::new(street)));
            action.as_ref().borrow_mut().execute(app.data_mut());

            app.plugin_mut(move |redo: &mut plugin_undo_redo::Redo<Map>| {
                redo.clear();
            });

            app.plugin_mut(move |undo: &mut plugin_undo_redo::Undo<Map>| {
                undo.push(Rc::clone(&action));
            });
        }

        false
    }
}

#[cfg(test)]
mod tests {}
