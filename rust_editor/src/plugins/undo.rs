use rust_macro::editor_plugin;

use crate::{actions::Action, ui::{app::EditorError, toolbar::ToolbarPosition}, keys};

use super::plugin::{Plugin, PluginWithOptions};

#[editor_plugin(skip)]
pub struct Undo<Data> {
    #[option(skip)]
    pub stack: Vec<Box<dyn Action<Data>>>,
}

impl<T> Undo<T> {
    pub fn push(&mut self, action: Box<dyn Action<T>>) {
        self.stack.push(action);
    }
}

impl<Data> Plugin<Data> for Undo<Data> where Data: Default + 'static {
    fn startup(&mut self, editor: &mut App<Data>) -> Result<(), EditorError> {
        editor.add_shortkey::<Undo<Data>>(keys!["Control", "z"])?;

        let toolbar =
            editor.get_or_add_toolbar("primary.undo_redo", ToolbarPosition::Left)?;

        let enabled = Rc::clone(&self.__enabled);

        toolbar.add_toggle_button(
            "undo",
            "mumumu",
            "Undo".to_string(),
            move || *enabled.as_ref().borrow(),
            move || EditorMessages::ActivatePlugin(Undo::<Data>::identifier()),
        )?;

        Ok(())
    }    
}
