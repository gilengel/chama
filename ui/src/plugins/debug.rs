use geo::{prelude::Centroid, Coordinate};
use rust_editor::{
    plugins::plugin::Plugin,
    ui::{
        app::{EditorError, Shortkey},
        toolbar::ToolbarPosition,
    }, gizmo::Id, input::keyboard::Key,
};
use rust_macro::editor_plugin;
use web_sys::CanvasRenderingContext2d;

use crate::map::{map::Map, intersection::Side};

#[editor_plugin(skip, specific_to=Map)]
pub struct Debug {}

impl Plugin<Map> for Debug {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        editor.add_shortkey::<Debug>(vec![Key::Ctrl, Key::U])?;

        let toolbar = editor.get_or_add_toolbar("primary.actions", ToolbarPosition::Left)?;

        let enabled = Rc::clone(&self.__enabled);
        toolbar.add_toggle_button(
            "info",
            "debug",
            "Show/Hide debug information".to_string(),
            move || *enabled.as_ref().borrow(),
            || EditorMessages::ShortkeyPressed(vec![Key::Ctrl, Key::U]),
        )?;

        Ok(())
    }

    fn shortkey_pressed(&mut self, key: &Shortkey, _: &Context<App<Map>>, _: &mut App<Map>) {
        if *key == vec![Key::Ctrl, Key::U] {
            let mut enabled = self.__enabled.borrow_mut();
            *enabled = !*enabled;
        }
    }

    fn render(&self, context: &CanvasRenderingContext2d, editor: &App<Map>) {
        let data = editor.data();
        for (_, street) in data.streets() {
            let mut owned_string: String = format!("{} ->", &street.id().to_string()[..2]);

        
            match &street.get_previous(Side::Left) {
                Some(l) => owned_string.push_str(&format!("{},", &l.to_string()[..2])),
                None => owned_string.push_str("#,"),
            }
            match &street.get_previous(Side::Right) {
                Some(l) => owned_string.push_str(&format!("{},", &l.to_string()[..2])),
                None => owned_string.push_str("#,"),
            }
            match &street.get_next(Side::Left) {
                Some(l) => owned_string.push_str(&format!("{},", &l.to_string()[..2])),
                None => owned_string.push_str("#,"),
            }
            match &street.get_next(Side::Right) {
                Some(l) => owned_string.push_str(&format!("{},", &l.to_string()[..2])),
                None => owned_string.push_str("#"),
            }
            
    
            if let Some(position) = street.polygon().exterior().centroid() {
                context.set_fill_style(&"#FFFFFF".into());
                context.fill_text(&owned_string, position.x(), position.y()).unwrap();
            }
    
            context.begin_path();
            let mut it = street.polygon().exterior().points();
            let start: Coordinate<f64> = it.next().unwrap().into();
            let width = street.width();
            let p1 = start + street.norm() * (street.length() - 5.0);
            let _p = start + street.norm() * (street.length() - width + 5.0);
            let p2 = _p + street.perp() * (-width / 2.0 + 5.0);
            let p3 = _p + street.perp() * (width / 2.0 - 5.0);
            context.move_to(p1.x, p1.y);
            context.line_to(p2.x, p2.y);
            context.line_to(p3.x, p3.y);
    
            context.close_path();
    
            context.save();
    
            context.set_stroke_style(&"#FFFFFF".into());
            context.stroke();
            context.restore();
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
    use crate::plugins::debug::Debug;

    #[test]
    fn integration_startup_adds_shortcut() {
        let mut app = App::<Map>::default();

        let mut plugin = Debug::default();
        plugin.startup(&mut app).unwrap();

        assert!(app.has_shortkey(vec![Key::Ctrl, Key::U]))
    }

    #[test]

    fn integration_startup_adds_toolbar_button() {
        let mut app = App::<Map>::default();

        let mut plugin = Debug::default();
        plugin.startup(&mut app).unwrap();

        let toolbar = app
            .get_or_add_toolbar("primary.actions", ToolbarPosition::Left)
            .unwrap();

        assert!(toolbar.has_button("debug"));
    }
}
