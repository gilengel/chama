use rust_macro::editor_plugin;

use rust_editor::{
    actions::Action,
    input::keyboard::Key,
    ui::{
        app::{EditorError, Shortkey},
        toolbar::ToolbarPosition,
    },
};

use rust_editor::plugin::Plugin;

#[editor_plugin(skip)]
pub struct Undo<Data> {
    #[option(skip)]
    pub stack: Vec<Rc<RefCell<dyn Action<Data>>>>,
}

impl<T> Undo<T> {
    pub fn push<S>(&mut self, action: Rc<RefCell<S>>)
    where
        S: Action<T> + Sized + 'static,
    {
        self.stack.push(action);
    }

    pub fn push_generic(&mut self, action: Rc<RefCell<dyn Action<T>>>) {
        self.stack.push(action);
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

impl<Data> Plugin<Data> for Undo<Data>
where
    Data: Default + 'static,
{
    fn startup(&mut self, editor: &mut App<Data>) -> Result<(), EditorError> {
        editor.add_shortkey::<Undo<Data>>(vec![Key::Ctrl, Key::Z])?;

        let toolbar = editor.get_or_add_toolbar("primary.undo_redo", ToolbarPosition::Left)?;

        toolbar.add_toggle_button(
            "undo",
            "undo",
            "Undo".to_string(),
            || false,
            || EditorMessages::ShortkeyPressed(vec![Key::Ctrl, Key::Z]),
        )?;

        Ok(())
    }

    fn shortkey_pressed(&mut self, key: &Shortkey, _: &Context<App<Data>>, editor: &mut App<Data>) {
        if *key == vec![Key::Ctrl, Key::Z] {
            if let Some(action) = self.stack.pop() {
                action.borrow_mut().undo(editor.data_mut());

                editor.plugin_mut(|redo: &mut super::Redo<Data>| {
                    redo.push_generic(Rc::clone(&action));
                });
            }
        }
    }
}
