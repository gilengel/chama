use rust_editor::{
    keys,
    plugins::plugin::Plugin,
    store::Store,
    ui::{
        app::{EditorError, Shortkey},
        toolbar::ToolbarPosition,
    },
};
use rust_macro::editor_plugin;

use crate::map::map::Map;

#[editor_plugin(skip, specific_to=Map)]
pub struct Load {}

impl Plugin<Map> for Load {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        editor.add_shortkey::<Load>(keys!["Control", "o"])?;

        let toolbar = editor.get_or_add_toolbar("primary.actions", ToolbarPosition::Left)?;
        toolbar.add_toggle_button(
            "open_in_browser",
            "load",
            "Load".to_string(),
            || false,
            || EditorMessages::ShortkeyPressed(keys!["Control", "o"]),
        )?;

        Ok(())
    }

    fn shortkey_pressed(&mut self, key: &Shortkey, _: &Context<App<Map>>, editor: &mut App<Map>) {
        if *key == keys!["Control", "o"] {
            if let Some(store) = Store::new("map_editor") {
                if let Some(data) = store.fetch_local_storage() {
                    editor.set_data(data);
                }
            }
        }
    }
}
