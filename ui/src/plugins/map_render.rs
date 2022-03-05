use rust_editor::{plugins::plugin::Plugin};
use rust_macro::editor_plugin;
use web_sys::CanvasRenderingContext2d;

use crate::map::map::Map;

#[editor_plugin(skip, specific_to=Map)]
pub struct MapRender {

}

impl Plugin<Map> for MapRender {
    fn render(&self, context: &CanvasRenderingContext2d, data: &Map) {
        for (_, district) in data.districts() {
            district.render(context).unwrap();
        }

        for (_, street) in data.streets() {
            street.render(context).unwrap();
        }

        for (_, intersection) in data.intersections() {
            intersection.render(&context).unwrap();
        }        
    }
}