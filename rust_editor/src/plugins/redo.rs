use rust_macro::editor_plugin;

use crate::{actions::Action, ui::{app::EditorError, toolbar::ToolbarPosition}, keys};

use super::plugin::{Plugin, PluginWithOptions};

#[editor_plugin(skip)]
pub struct Redo<Data> {
    #[option(skip)]
    pub stack: Vec<Box<dyn Action<Data>>>,
}

impl<T> Redo<T> {
    pub fn push(&mut self, action: Box<dyn Action<T>>) {
        self.stack.push(action);
    }
}

impl<Data> Plugin<Data> for Redo<Data>
where
    Data: Default + 'static,
{
    fn startup(&mut self, editor: &mut App<Data>) -> Result<(), EditorError> {
        editor.add_shortkey::<Redo<Data>>(keys!["Control", "y"])?;

        let toolbar =
            editor.get_or_add_toolbar("primary.undo_redo", ToolbarPosition::Left)?;

        toolbar.add_toggle_button(
            "redo",
            "mumumu",
            "Redo".to_string(),
            || false,
            move || EditorMessages::ActivatePlugin(Redo::<Data>::identifier()),
        )?;

        Ok(())
    }        
}
