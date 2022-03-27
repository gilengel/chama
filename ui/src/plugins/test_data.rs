use geo::Coordinate;
use rust_editor::{actions::Action, plugins::plugin::Plugin, ui::app::EditorError};
use rust_macro::editor_plugin;

use crate::map::map::Map;

use super::create_freeform_street::CreateFreeFormStreetAction;

#[editor_plugin(skip, specific_to=Map)]
pub struct TestData {}

impl Plugin<Map> for TestData {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        /*
        fn r2d(deg: f64) -> f64 {
            deg * PI / 180.
        }

        let num_pts = 10;
        let radius = 256.;
        let center = Coordinate { x: 1024., y: 512. };
        let mut points = Vec::<Coordinate<f64>>::with_capacity(num_pts);
        for i in 0..num_pts {
            let rad = r2d(i as f64 / num_pts as f64 * 360.);
            points.push(center + Coordinate { x: f64::cos(rad) * radius, y: f64::sin(rad) * radius }
        }
        //points.push(*points.first().unwrap());

        points.push(Coordinate { x: 512., y: 512. };

        let action = Rc::new(RefCell::new(CreateFreeFormStreetAction::new(points)));
        action.borrow_mut().execute(editor.data_mut());

        editor.plugin_mut(move |undo: &mut rust_editor::plugins::undo::Undo<Map>| {
            undo.push(Rc::clone(&action));
        };

        */

        let points = vec![
            Coordinate { x: 196.0, y: 433.0 },
            Coordinate { x: 635.0, y: 433.0 },
            Coordinate { x: 785.0, y: 439.0 },
            Coordinate { x: 915.0, y: 435.0 },
            Coordinate {
                x: 1095.0,
                y: 415.0,
            },
            Coordinate {
                x: 1283.0,
                y: 385.0,
            },
            Coordinate {
                x: 1403.0,
                y: 355.0,
            },
            Coordinate {
                x: 1501.0,
                y: 305.0,
            },
            Coordinate {
                x: 1523.0,
                y: 268.0,
            },
            Coordinate {
                x: 1503.0,
                y: 209.0,
            },
            Coordinate {
                x: 1439.0,
                y: 155.0,
            },
            Coordinate {
                x: 1319.0,
                y: 109.0,
            },
            Coordinate {
                x: 1181.0,
                y: 103.0,
            },
            Coordinate {
                x: 1009.0,
                y: 133.0,
            },
            Coordinate { x: 773.0, y: 219.0 },
            Coordinate { x: 573.0, y: 329.0 },
            Coordinate { x: 573.0, y: 533.0 },
            //Coordinate { x: 338.0, y: 600.0 },
        ];
        let action = Rc::new(RefCell::new(CreateFreeFormStreetAction::new(points)));
        action.borrow_mut().execute(editor.data_mut());

        Ok(())
    }
}
