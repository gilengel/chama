use geo::Coordinate;
use rust_editor::{actions::Action, plugins::plugin::Plugin, ui::app::EditorError};
use rust_macro::editor_plugin;
use uuid::Uuid;

use crate::map::{map::Map, actions::street::create::CreateStreet};


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

        //let points = vec![Coordinate { x: 262.0, y: 230.0 }, Coordinate { x: 372.0, y: 183.0 }, Coordinate { x: 448.0, y: 160.0 }, Coordinate { x: 602.0, y: 136.0 }, Coordinate { x: 692.0, y: 134.0 }, Coordinate { x: 796.0, y: 148.0 }, Coordinate { x: 936.0, y: 186.0 }, Coordinate { x: 1104.0, y: 257.0 }, Coordinate { x: 1290.0, y: 365.0 }, Coordinate { x: 1366.0, y: 432.0 }, Coordinate { x: 1425.0, y: 495.0 }, Coordinate { x: 1535.0, y: 620.0 }, Coordinate { x: 1557.0, y: 655.0 }, Coordinate { x: 1557.0, y: 682.0 }, Coordinate { x: 1545.0, y: 706.0 }, Coordinate { x: 1444.0, y: 761.0 }, Coordinate { x: 1370.0, y: 784.0 }, Coordinate { x: 1225.0, y: 804.0 }, Coordinate { x: 1031.0, y: 820.0 }, Coordinate { x: 903.0, y: 824.0 }, Coordinate { x: 661.0, y: 816.0 }, Coordinate { x: 578.0, y: 799.0 }, Coordinate { x: 458.0, y: 735.0 }, Coordinate { x: 400.0, y: 694.0 }, Coordinate { x: 363.0, y: 650.0 }, Coordinate { x: 332.0, y: 589.0 }, Coordinate { x: 314.0, y: 504.0 }, Coordinate { x: 316.0, y: 450.0 }, Coordinate { x: 333.0, y: 372.0 }, Coordinate { x: 355.0, y: 315.0 }, Coordinate { x: 439.0, y: 191.0 }, Coordinate { x: 544.0, y: 103.0 }, Coordinate { x: 602.0, y: 68.0 }, Coordinate { x: 655.0, y: 46.0 }, Coordinate { x: 710.0, y: 31.0 }, Coordinate { x: 808.0, y: 20.0 }, Coordinate { x: 902.0, y: 18.0 }, Coordinate { x: 946.0, y: 23.0 }, Coordinate { x: 1125.0, y: 65.0 }, Coordinate { x: 1215.0, y: 93.0 }, Coordinate { x: 1489.0, y: 205.0 }, Coordinate { x: 1556.0, y: 244.0 }, Coordinate { x: 1608.0, y: 288.0 }, Coordinate { x: 1618.0, y: 300.0 }, Coordinate { x: 1653.0, y: 393.0 }, Coordinate { x: 1396.0, y: 398.0 }, Coordinate { x: 1184.0, y: 414.0 }, Coordinate { x: 930.0, y: 458.0 }, Coordinate { x: 699.0, y: 489.0 }, Coordinate { x: 397.0, y: 509.0 }, Coordinate { x: 246.0, y: 530.0 }, Coordinate { x: 180.0, y: 547.0 }, Coordinate { x: 162.0, y: 561.0 }, Coordinate { x: 148.0, y: 580.0 }, Coordinate { x: 134.0, y: 647.0 }, Coordinate { x: 142.0, y: 748.0 }, Coordinate { x: 152.0, y: 779.0 }, Coordinate { x: 179.0, y: 816.0 }, Coordinate { x: 211.0, y: 845.0 }, Coordinate { x: 238.0, y: 857.0 }, Coordinate { x: 418.0, y: 882.0 }, Coordinate { x: 480.0, y: 885.0 }, Coordinate { x: 507.0, y: 875.0 }, Coordinate { x: 582.0, y: 802.0 }, Coordinate { x: 635.0, y: 717.0 }, Coordinate { x: 690.0, y: 589.0 }, Coordinate { x: 787.0, y: 461.0 }, Coordinate { x: 954.0, y: 268.0 }, Coordinate { x: 1039.0, y: 199.0 }, Coordinate { x: 1182.0, y: 103.0 }, Coordinate { x: 1399.0, y: 12.0 }];
        //let action = Rc::new(RefCell::new(CreateFreeFormStreetAction::new(points)));
        //action.borrow_mut().execute(editor.data_mut());

        let map = editor.data_mut();
        CreateStreet::new(Coordinate { x: 256., y: 0.}, Coordinate { x: 256., y: 512. }, Uuid::new_v4()).execute(map);
        CreateStreet::new(Coordinate { x: 256., y: 512.}, Coordinate { x: 256., y: 1024. }, Uuid::new_v4()).execute(map);

        let mut ids = Vec::<Uuid>::with_capacity(4);
        for i in 0..4 {
            let id = Uuid::new_v4();
            ids.push(id);
            CreateStreet::new(Coordinate { x: 256. + i as f64 * 256., y: 512.}, Coordinate { x: 256. + (i+1) as f64 * 256., y: 512. }, id).execute(map);
        }

        CreateStreet::new(Coordinate { x: 1024. + 256., y: 0.}, Coordinate { x: 1024. +256., y: 512. }, Uuid::new_v4()).execute( map);
        CreateStreet::new(Coordinate { x: 1024. + 256., y: 512.}, Coordinate { x: 1024. + 256., y: 1024. }, Uuid::new_v4()).execute( map);

        Ok(())
    }
}
