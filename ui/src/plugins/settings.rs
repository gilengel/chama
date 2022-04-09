use rust_editor::{
    keys,
    plugins::plugin::Plugin,
    ui::{
        app::{EditorError, Shortkey},
        panel::Panel,
        toolbar::ToolbarPosition,
    },
};
use rust_macro::editor_plugin;
use wasm_bindgen::JsCast;

use crate::map::map::Map;

#[editor_plugin(skip, specific_to=Map)]
pub struct Settings {
    #[option(skip)]
    visible: Rc<RefCell<bool>>,
}

impl Plugin<Map> for Settings {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        editor.add_shortkey::<Settings>(keys!["Control", "m"])?;

        let toolbar = editor.get_or_add_toolbar("primary.actions", ToolbarPosition::Left)?;

        let visible = Rc::clone(&self.visible);
        toolbar.add_toggle_button(
            "settings",
            "settings",
            "Settings".to_string(),
            move || !*visible.as_ref().borrow(),
            || EditorMessages::ShortkeyPressed(keys!["Control", "m"]),
        )?;

        Ok(())
    }

    fn shortkey_pressed(
        &mut self,
        key: &Shortkey,
        _ctx: &Context<App<Map>>,
        _editor: &mut App<Map>,
    ) {
        if *key == keys!["Control", "m"] {
            let mut visible = self.visible.borrow_mut();
            *visible = !(*visible);

            let window = web_sys::window().expect("no global `window` exists");
            let document = window.document().expect("should have a document on window");
            let body = document.body().expect("document should have a body");

            let dialogs = body.get_elements_by_class_name("panel");
            if let Some(settings) = dialogs.item(0) {
                let settings = settings.dyn_into::<web_sys::HtmlElement>().unwrap();
                let value = if *visible { "hidden" } else { "visible" };
                settings.style().set_property("visibility", value).unwrap();
            }
        }
    }

    fn editor_elements(&mut self, ctx: &Context<App<Map>>, editor: &App<Map>) -> Vec<Html> {
        let plugins = editor.plugins();
        let mut elements: Vec<Html> = Vec::new();

        elements.push(html! {
        <Panel>
            {
                for plugins.filter(|(id, _)| **id != "Settings").map(|(_, plugin)| {
                    plugin.as_ref().borrow().view_options(ctx)
                })
            }
        </Panel>
        });

        elements
    }
}

#[cfg(test)]
mod tests {
    use rust_editor::plugins::plugin::Plugin;
    use rust_editor::ui::app::App;
    use rust_editor::ui::toolbar::ToolbarPosition;

    use crate::map::map::Map;
    use crate::plugins::settings::Settings;

    #[test]
    fn integration_startup_adds_toolbar_button() {
        let mut app = App::<Map>::default();

        let mut plugin = Settings::default();
        plugin.startup(&mut app).unwrap();

        let toolbar = app
            .get_or_add_toolbar("primary.actions", ToolbarPosition::Left)
            .unwrap();

        assert!(toolbar.has_button("settings"));
    }
}
