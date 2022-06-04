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
        
        // gap left
        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: 537., y: 303. },
            coord! { x: 528., y: 263. },
            coord! { x: 507., y: 193. },
        ]));
        action.execute(editor.data_mut()); 

        // bottom
        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: 378., y: 400. },
            coord! { x: 451., y: 421. },
            coord! { x: 550., y: 429. },
            coord! { x: 702., y: 448. },
        ]));
        action.execute(editor.data_mut()); 

        // gap right
        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: 672., y: 190. },
            coord! { x: 671., y: 208. },
            coord! { x: 646., y: 290. },
        ]));
        action.execute(editor.data_mut()); 

        // top left
        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: 507., y: 193. },
            coord! { x: 472., y: 188. },
            coord! { x: 390., y: 184. },
        ]));
        action.execute(editor.data_mut()); 

        // top right
        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: 801., y: 247. },
            coord! { x: 751., y: 231. },
            coord! { x: 672., y: 190. },
        ]));
        action.execute(editor.data_mut()); 

        // left
        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: 390., y: 184. },
            coord! { x: 291., y: 206. },
            coord! { x: 378., y: 400. },
        ]));
        action.execute(editor.data_mut()); 

        // right
        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: 702., y: 448. },
            coord! { x: 744., y: 362. },
            coord! { x: 801., y: 247. },
        ]));      
        action.execute(editor.data_mut());


        // gap bottom
        let mut action = CreateFreeFormStreetAction::new(LineString::new(vec![
            coord! { x: 646., y: 290. },
            coord! { x: 555., y: 301. },
            coord! { x: 537., y: 303. },
        ]));
        action.execute(editor.data_mut()); 
                
        Ok(())
    }
}