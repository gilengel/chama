use rust_editor::input::keyboard::Key;
use rust_editor::ui::dialog::Dialog;
use rust_editor::{
    plugin::Plugin,
    ui::{
        app::{EditorError, Shortkey},
        toolbar::ToolbarPosition,
    },
};
use rust_macro::editor_plugin;

use crate::map::map::Map;

#[editor_plugin(skip, specific_to=Map)]
pub struct New {
    #[option(skip)]
    dialog_visible: Rc<RefCell<bool>>,
}

impl Plugin<Map> for New {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        editor.add_shortkey::<New>(vec![Key::Ctrl, Key::N])?;

        let toolbar = editor.get_or_add_toolbar("primary.actions", ToolbarPosition::Left)?;

        toolbar.add_toggle_button(
            "restore_page",
            "new",
            "New".to_string(),
            || false,
            || EditorMessages::ShortkeyPressed(vec![Key::Ctrl, Key::N]),
        )?;

        Ok(())
    }

    fn shortkey_pressed(&mut self, key: &Shortkey, ctx: &Context<App<Map>>, editor: &mut App<Map>) {
        if *key == vec![Key::Ctrl, Key::N] {
            let mut dialog_visible = self.dialog_visible.borrow_mut();
            if *dialog_visible {
                let map = editor.data_mut();
                map.clear();

                editor.plugin_mut(move |redo: &mut plugin_undo_redo::Redo<Map>| {
                    redo.clear();
                });

                editor.plugin_mut(move |undo: &mut plugin_undo_redo::undo::Undo<Map>| {
                    undo.clear();
                });
            }

            *dialog_visible = !*dialog_visible;
            ctx.link().send_message(EditorMessages::UpdateElements());
        }
    }

    fn editor_elements(&mut self, ctx: &Context<App<Map>>, _: &App<Map>) -> Vec<Html> {
        let mut elements: Vec<Html> = Vec::new();

        let dialog_visible = Rc::clone(&self.dialog_visible);
        if *self.dialog_visible.as_ref().borrow() {
            let cancel = ctx.link().callback(move |_| {
                *dialog_visible.borrow_mut() = false;
                EditorMessages::UpdateElements()
            });
            let discard = ctx
                .link()
                .callback(move |_| EditorMessages::ShortkeyPressed(vec![Key::Ctrl, Key::N]));

            elements.push(html! {
            <Dialog title="Save changes before closing?">
                <button onclick={discard}>{"Discard"}</button>
                <button onclick={cancel}>{"Cancel"}</button>
                <button>{"Save"}</button>
            </Dialog>
            });
        }

        elements
    }
}

#[cfg(test)]
mod tests {
    use rust_editor::input::keyboard::Key;
    use rust_editor::plugins::plugin::Plugin;
    use rust_editor::ui::app::App;
    use rust_editor::ui::toolbar::ToolbarPosition;

    use crate::map::map::Map;
    use crate::plugins::new::New;

    #[test]
    fn integration_startup_adds_shortcut() {
        let mut app = App::<Map>::default();

        let mut plugin = New::default();
        plugin.startup(&mut app).unwrap();

        assert!(app.has_shortkey(vec![Key::Ctrl, Key::N]))
    }

    #[test]

    fn integration_startup_adds_toolbar_button() {
        let mut app = App::<Map>::default();

        let mut plugin = New::default();
        plugin.startup(&mut app).unwrap();

        let toolbar = app
            .get_or_add_toolbar("primary.actions", ToolbarPosition::Left)
            .unwrap();

        assert!(toolbar.has_button("new"));
    }
}
