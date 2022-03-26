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
pub struct Save {}

impl Plugin<Map> for Save {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        editor.add_shortkey::<Save>(keys!["Control", "s"])?;

        let toolbar = editor.get_or_add_toolbar("primary.actions", ToolbarPosition::Left)?;

        toolbar.add_toggle_button(
            "save",
            "save",
            "Save".to_string(),
            || false,
            || EditorMessages::ShortkeyPressed(keys!["Control", "s"]),
        )?;

        Ok(())
    }

    fn shortkey_pressed(&mut self, key: &Shortkey, _: &Context<App<Map>>, editor: &mut App<Map>) {
        if *key == keys!["Control", "s"] {
            if let Some(store) = Store::new("map_editor") {
                store.sync_local_storage(editor.data()).unwrap();
            }
        }
    }
}
