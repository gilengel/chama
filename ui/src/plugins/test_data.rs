use geo::{coord, prelude::EuclideanDistance, Coordinate, Point};
use rust_editor::{actions::Action, log, plugins::plugin::Plugin, ui::app::EditorError};
use rust_macro::editor_plugin;
use uuid::Uuid;

use crate::map::{actions::street::create::CreateStreet, map::Map};

use super::create_freeform_street::CreateFreeFormStreetAction;

#[editor_plugin(skip, specific_to=Map)]
pub struct TestData {}

impl Plugin<Map> for TestData {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        fn square(size: f64, start: Coordinate<f64>, editor: &mut App<Map>) {
            let action = Rc::new(RefCell::new(CreateFreeFormStreetAction::new(vec![
                start,
                start + coord! { x: size, y: 0.},
                start + coord! { x: size, y: size},
                start + coord! { x: 0., y: size},
                start,
            ])));

            action.borrow_mut().execute(editor.data_mut());
        }

        fn straight_line(
            start: Coordinate<f64>,
            end: Coordinate<f64>,
            segments: u32,
            editor: &mut App<Map>,
        ) {
            let start: Point<f64> = start.into();
            let end: Point<f64> = end.into();
            let vec = end - start;

            let length = start.euclidean_distance(&end);
            let norm: Point<f64> = Point::new(vec.x() / length, vec.y() / length);

            let factor = length / segments as f64;
            let mut pts: Vec<Coordinate<f64>> = vec![];
            for i in 0..segments+1 {
                pts.push((start + norm * i as f64 * factor).into());
            }

            let action = Rc::new(RefCell::new(CreateFreeFormStreetAction::new(pts)));

            action.borrow_mut().execute(editor.data_mut());
        }

        square(512.0, coord! { x: 0., y: 0.}, editor);

        log!("\n\n===========================\n\n");

        //square(512.0, coord! { x: 0., y: 0.}, editor);
        //square(256.0, coord! { x: 0. + 512., y: 256.}, editor);

        
        straight_line(
            coord! { x: 512. - 10., y: 512.},
            coord! { x: 512. + 10., y: 128.},
            1,
            editor,
        );

        log!("\n\n===========================\n\n");

        for (_, street) in editor.data().streets() {
            log!("{:?} {:?}", street.start().x_y(), street.end().x_y());
        }

        Ok(())
    }
}
