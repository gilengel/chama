use crate::snackbar::Snackbar;
use rust_editor::log;
use rust_editor::plugin::Plugin;
use rust_editor::ui::app::{EditorError, Shortkey};
use rust_macro::editor_plugin;

#[cfg(feature = "snackbar")]
use snackbar::{SnackbarAction, SnackbarPosition};

#[cfg(feature = "snackbar")]
pub mod snackbar;

#[editor_plugin(skip)]
pub struct ComponentsPlugin {
    #[option(skip)]
    elements: Vec<Html>,
}

pub enum ComponentMessage {
    #[cfg(feature = "snackbar")]
    ShowSnackbar(
        &'static str,
        Option<SnackbarPosition>,
        Option<SnackbarAction>,
    ),
}

unsafe impl Send for ComponentsPlugin {}

impl ComponentsPlugin {
    #[cfg(feature = "snackbar")]
    pub fn show_snackbar(
        &mut self,
        text: &'static str,
        position: Option<SnackbarPosition>,
        _: Option<SnackbarAction>,
    ) {
        let action = SnackbarAction {
            label: "Retry".to_string(),
            callback: Rc::new(|| log!("Click on retry")),
        };

        let position = position.unwrap_or_default();
        let snackbar = html! {
            <Snackbar message={text} action={action} position={position} />
        };

        self.elements.push(snackbar);
    }
}

impl<Data> Plugin<Data> for ComponentsPlugin
where
    Data: Default + 'static,
{
    fn startup(&mut self, _: &mut App<Data>) -> Result<(), EditorError> {
        Ok(())
    }

    fn shortkey_pressed(&mut self, _: &Shortkey, _: &Context<App<Data>>, _: &mut App<Data>) {}

    fn editor_elements(&mut self, _: &Context<App<Data>>, _: &App<Data>) -> Vec<Html> {
        self.elements.clone()
    }

    fn on_message(&mut self, message: Box<dyn Any>) {
        let message = message.as_ref().downcast_ref::<ComponentMessage>().unwrap();

        // TODO enable option
        match message {
            #[cfg(feature = "snackbar")]
            ComponentMessage::ShowSnackbar(text, position, _) => {
                self.show_snackbar(text, position.clone(), None)
            }
        }
    }
}
