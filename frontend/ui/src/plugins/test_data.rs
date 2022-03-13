use geo::Coordinate;
use rust_editor::{plugins::plugin::Plugin, ui::app::EditorError};
use rust_macro::editor_plugin;

use crate::map::map::Map;

#[editor_plugin(skip, specific_to=Map)]
pub struct TestData {}

impl Plugin<Map> for TestData {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        let data = editor.data_mut();

        data.create_street(&Coordinate { x: 200.0, y: 200.0 }, &Coordinate { x: 800.0, y: 200.0 }, 5.);
        data.create_street(&Coordinate { x: 800.0, y: 200.0 }, &Coordinate { x: 800.0, y: 800.0 }, 5.);
        data.create_street(&Coordinate { x: 800.0, y: 800.0 }, &Coordinate { x: 200.0, y: 800.0 },5.);
        data.create_street(&Coordinate { x: 200.0, y: 800.0 }, &Coordinate { x: 200.0, y: 200.0 }, 5.);


        Ok(())
    }
}
