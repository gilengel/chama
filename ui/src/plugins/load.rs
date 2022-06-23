use rust_editor::{
    input::keyboard::Key,
    plugin::Plugin,
    store::Store,
    ui::app::{EditorError, Shortkey},
};
use rust_macro::editor_plugin;

use crate::map::map::Map;
use plugin_toolbar::toolbar::ToolbarPosition;

#[editor_plugin(skip, specific_to=Map)]
pub struct Load {}

impl Plugin<Map> for Load {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        editor.add_shortkey::<Load>(vec![Key::Ctrl, Key::O])?;

        editor.plugin_mut(
            move |toolbar_plugin: &mut plugin_toolbar::ToolbarPlugin<Map>| {
                let toolbar = toolbar_plugin
                    .get_or_add_toolbar("primary.actions", ToolbarPosition::Left)
                    .unwrap();
                toolbar
                    .add_toggle_button(
                        "open_in_browser",
                        "load",
                        "Load".to_string(),
                        || false,
                        || EditorMessages::ShortkeyPressed(vec![Key::Ctrl, Key::O]),
                    )
                    .unwrap();
            },
        );

        Ok(())
    }

    fn shortkey_pressed(&mut self, key: &Shortkey, _: &Context<App<Map>>, editor: &mut App<Map>) {
        if *key == vec![Key::Ctrl, Key::O] {
            if let Some(store) = Store::new("map_editor") {
                if let Some(data) = store.fetch_local_storage() {
                    editor.set_data(data);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_editor::input::keyboard::Key;
    use rust_editor::plugins::plugin::Plugin;
    use rust_editor::ui::app::App;
    use rust_editor::ui::toolbar::ToolbarPosition;

    use crate::map::map::Map;
    use crate::plugins::load::Load;

    #[test]
    fn integration_startup_adds_shortcut() {
        let mut app = App::<Map>::default();

        let mut plugin = Load::default();
        plugin.startup(&mut app).unwrap();

        assert!(app.has_shortkey(vec![Key::Ctrl, Key::O]))
    }

    #[test]

    fn integration_startup_adds_toolbar_button() {
        let mut app = App::<Map>::default();

        let mut plugin = Load::default();
        plugin.startup(&mut app).unwrap();

        let toolbar = app
            .get_or_add_toolbar("primary.actions", ToolbarPosition::Left)
            .unwrap();

        assert!(toolbar.has_button("load"));
    }
}
