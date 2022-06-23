use rust_editor::plugin::Plugin;
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

        /*

        data.street_polygon.render(&Style::default(), &context).unwrap();

        let district_style = Style {
                border_width: 0,
                border_color: "#0000".to_string(),
                background_color: "#00FF00FF".to_string(),
            };

        for polys in &data.street_polygon {
            for district_polygon in polys.interiors() {
                //district_polygon.render(&district_style, &context).unwrap();
            }
        }
        */
    }
}
