use rust_macro::editor_plugin;

use crate::{actions::Action, ui::{app::{EditorError, Shortkey}, toolbar::ToolbarPosition}, input::keyboard::Key};

use super::plugin::{Plugin};

#[editor_plugin(skip)]
pub struct Redo<Data> {
    #[option(skip)]
    stack: Vec<Rc<RefCell<dyn Action<Data>>>>,
}

impl<T> Redo<T> {
    pub fn push<S>(&mut self, action: Rc<RefCell<S>>) where S : Action<T> + 'static {
        self.stack.push(action);
    }

    pub fn push_generic(&mut self, action: Rc<RefCell<dyn Action<T>>>) {
        self.stack.push(action);
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

impl<Data> Plugin<Data> for Redo<Data>
where
    Data: Default + 'static,
{
    fn startup(&mut self, editor: &mut App<Data>) -> Result<(), EditorError> {
        editor.add_shortkey::<Redo<Data>>(vec![Key::Ctrl, Key::Y])?;

        let toolbar =
            editor.get_or_add_toolbar("primary.undo_redo", ToolbarPosition::Left)?;

        toolbar.add_toggle_button(
            "redo",
            "redo",
            "Redo".to_string(),
            || false,
            || EditorMessages::ShortkeyPressed(vec![Key::Ctrl, Key::Y]),
        )?;

        Ok(())
    }
    
    fn shortkey_pressed(&mut self, key: &Shortkey, _: &Context<App<Data>>, editor: &mut App<Data>) {
        if *key == vec![Key::Ctrl, Key::Y] {
            if let Some(action) = self.stack.pop() {
                action.borrow_mut().redo(editor.data_mut());

                editor.plugin_mut(|undo: &mut crate::plugins::undo::Undo<Data>| {
                    undo.push_generic(Rc::clone(&action));
                });
            }
        }

    }        
}
