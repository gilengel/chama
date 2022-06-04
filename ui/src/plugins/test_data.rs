use geo::{coord, LineString};
use rust_editor::{actions::Action, plugins::plugin::Plugin, ui::app::EditorError, log};
use rust_macro::editor_plugin;
use uuid::Uuid;

use crate::map::{map::Map};

use super::create_freeform_street::CreateFreeFormStreetAction;


#[editor_plugin(skip, specific_to=Map)]
pub struct TestData {}

impl Plugin<Map> for TestData {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        
        /*
        let id = Uuid::new_v4();

        let start_x = 256.;
        let end_x = 256. * 3.;
        let start_y = 512. -128. * 2.;
        let end_y = 512.;

        let gap_x = 0.4;
        let gap_y = 0.1;

        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: start_x, y: end_y },
            coord! { x: start_x + (end_x - start_x) / 2., y: end_y },
            coord! { x: end_x, y: end_y },  
        ]));
        action.execute(editor.data_mut());    

        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: start_x, y: end_y },
            coord! { x: start_x, y: start_y + (end_y - start_y) / 2. },
            coord! { x: start_x, y: start_y },  
        ]));        
        action.execute(editor.data_mut()); 

        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: end_x, y: end_y },
            coord! { x: end_x, y: start_y + (end_y - start_y) / 2. },
            coord! { x: end_x, y: start_y },  
        ]));        
        action.execute(editor.data_mut()); 

        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: start_x, y: start_y },
            coord! { x: start_x + (end_x - start_x) * (1. - gap_x) / 4., y: start_y },
            coord! { x: start_x + (end_x - start_x) * (1. - gap_x) / 2., y: start_y },  
        ]));
        action.execute(editor.data_mut());   

        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: start_x + (end_x - start_x) * (1. - gap_x) / 2., y: start_y },
            coord! { x: start_x + (end_x - start_x) * (1. - gap_x) / 2., y: start_y + (end_y - start_y) * 0.2 },
            coord! { x: start_x + (end_x - start_x) * (1. - gap_x) / 2., y: start_y + (end_y - start_y) * 0.4 },  
        ]));        
        action.execute(editor.data_mut());  
        
        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: start_x + (end_x - start_x) * (1. - gap_x) / 2., y: start_y + (end_y - start_y) * 0.4 },
            coord! { x: start_x + (end_x - start_x) * 0.5, y: start_y + (end_y - start_y) * 0.4 },
            coord! { x: start_x + (end_x - start_x) * (1. - gap_x), y: start_y + (end_y - start_y) * 0.4 },  
        ]));
        action.execute(editor.data_mut());  
        
        
        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: start_x + (end_x - start_x) * (1. - gap_x) / 2., y: start_y },
            coord! { x: start_x + (end_x - start_x) * (1. - gap_x) / 2., y: start_y + (end_y - start_y) * 0.2 },
            coord! { x: start_x + (end_x - start_x) * (1. - gap_x) / 2., y: start_y + (end_y - start_y) * 0.4 },  
        ]));        
        action.execute(editor.data_mut()); 

        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: start_x + (end_x - start_x) * 0.6, y: start_y },
            coord! { x: start_x + (end_x - start_x) * 0.8, y: start_y },
            coord! { x: end_x, y: start_y },  
        ]));
        action.execute(editor.data_mut()); 
        */ 
        
        Ok(())
    }
}