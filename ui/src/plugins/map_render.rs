use geo::{polygon, Polygon};
use geo_booleanop::boolean::BooleanOp;
use rust_editor::{plugins::plugin::Plugin, renderer::PrimitiveRenderer, style::Style, log};
use rust_macro::editor_plugin;
use web_sys::CanvasRenderingContext2d;

use crate::map::map::Map;


#[editor_plugin(skip, specific_to=Map)]
pub struct MapRender {}

impl Plugin<Map> for MapRender {
    fn render(&self, context: &CanvasRenderingContext2d, editor: &App<Map>) {
        
        let data = editor.data();
        for (_, district) in data.districts() {
            district.render(context).unwrap();
        }

        for (_, street) in data.streets() {
            street.render(context).unwrap();
        }

        for (_, intersection) in data.intersections() {
            intersection.render(&context).unwrap();
        }

        data.street_polygon.render(&Style::default(), &context).unwrap();
    }
}
