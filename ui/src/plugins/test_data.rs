use geo::Coordinate;
use rust_editor::{actions::Action, plugins::plugin::Plugin, ui::app::EditorError};
use rust_macro::editor_plugin;

use crate::map::map::Map;

use super::create_freeform_street::CreateFreeFormStreetAction;

#[editor_plugin(skip, specific_to=Map)]
pub struct TestData {}

impl Plugin<Map> for TestData {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        let points = vec![
            Coordinate { x: 256., y: 512. },
            Coordinate { x: 1024., y: 512. },
            Coordinate { x: 1024., y: 256. },
            Coordinate { x: 512., y: 256. },
            Coordinate { x: 512., y: 1024. },
        ];
        let action = Rc::new(RefCell::new(CreateFreeFormStreetAction::new(points)));
        action.borrow_mut().execute(editor.data_mut());

        editor.plugin_mut(move |undo: &mut rust_editor::plugins::undo::Undo<Map>| {
            undo.push(Rc::clone(&action));
        });

        Ok(())
    }
}
