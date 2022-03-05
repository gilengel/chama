use std::collections::HashMap;

use geo::{
    coords_iter::CoordsIter, prelude::Centroid, simplify::Simplify, winding_order::Winding,
    Coordinate, LineString, Point, Polygon,
};
use rust_editor::{
    gizmo::{Id, SetPosition},
    plugins::{plugin::PluginWithOptions},
    renderer::PrimitiveRenderer,
    style::Style,
    system::System,
    InformationLayer,
};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{
    map::{
        district::create_district_for_street, intersection::Intersection, map::Map, street::Street,
    },
    Modes,
};

pub struct CreateFreeFormDistrictSystem {
    raw_points: Vec<Coordinate<f64>>,
    raw_point_style: Style,
    raw_polygon: Polygon<f64>,
    raw_polygon_style: Style,

    brush_active: bool,
}

impl Default for CreateFreeFormDistrictSystem {
    fn default() -> Self {
        CreateFreeFormDistrictSystem {
            raw_points: Vec::new(),
            raw_point_style: Style {
                border_width: 0,
                border_color: "#FFFFFF".to_string(),
                background_color: "#2A2A2B".to_string(),
            },
            raw_polygon: Polygon::new(LineString::from(Vec::<Coordinate<f64>>::new()), vec![]),
            raw_polygon_style: Style {
                border_width: 0,
                border_color: "#FFFFFF".to_string(),
                background_color: "rgba(30, 136, 229, 0.4)".to_string(),
            },
            brush_active: false,
        }
    }
}

impl CreateFreeFormDistrictSystem {
    fn create_intersection(&self, pos: &Point<f64>) -> Intersection {
        let mut intersection = Intersection::default();
        intersection.set_position(Coordinate {
            x: pos.x(),
            y: pos.y(),
        });

        intersection
    }

    pub fn transform_polygon_into_streets(&self, map: &mut Map) {
        let mut intersections: Vec<Uuid> = vec![Uuid::default()];

        // We skip the first coordinate since it is equal with the last one and we want to avoid
        // to add the intersection twice. After looping through all the points we simply overwrite
        // the placeholder at position 0 with the last element to make insertion of streets easier
        for point in self.raw_polygon.exterior().points_cw().into_iter().skip(1) {
            intersections.push(map.add_intersection(self.create_intersection(&point)));
        }
        intersections[0] = *intersections.last().unwrap();

        let mut street_id = Uuid::default();
        let mut it = intersections.iter().peekable();
        while let Some(current_id) = it.next() {
            if let Some(next_id) = it.peek() {
                let current_intersection = map.intersection(current_id).unwrap();
                let next_intersection = map.intersection(next_id).unwrap();

                let mut street = Street::default();
                street_id = street.id();
                street.set_start(&current_intersection);
                street.set_end(&next_intersection);

                map.add_street(street);

                if let Some(current_intersection) = map.intersection_mut(current_id) {
                    current_intersection.add_outgoing_street(&street_id);
                }

                if let Some(next_intersection) = map.intersection_mut(next_id) {
                    next_intersection.add_incoming_street(&street_id);
                }
            }
        }

        for intersection in intersections {
            map.update_intersection(&intersection);
        }

        let street = map.street(&street_id).unwrap();
        let side = street.get_side_of_position(&self.raw_polygon.centroid().unwrap().into());

        if let Some(district) = create_district_for_street(side, street_id, map) {
            map.add_district(district);
        }
    }
}

impl System<Map, Modes> for CreateFreeFormDistrictSystem {
    fn mouse_down(
        &mut self,
        _: Coordinate<f64>,
        button: u32,
        _: &mut Map,
        _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) {
        if button == 0 {
            self.raw_polygon.exterior_mut(|exterior| exterior.0.clear());
            self.raw_points.clear();

            self.brush_active = true;
        }
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _: &mut Map,
        _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) {
        if self.brush_active {
            self.raw_points.push(mouse_pos);

            self.raw_polygon
                .exterior_mut(|exterior| exterior.0 = self.raw_points.clone());
        }
    }

    fn mouse_up(
        &mut self,
        _: Coordinate<f64>,
        button: u32,
        map: &mut Map,

        _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) {
        if button == 0 {
            self.brush_active = false;
        }

        self.raw_polygon = self.raw_polygon.simplify(&8.0);

        self.transform_polygon_into_streets(map);
    }

    fn render(
        &self,
        map: &Map,
        context: &CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
        _plugins: &HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) -> Result<(), JsValue> {
        map.render(context, additional_information_layer)?;

        if self.brush_active {
            self.raw_polygon.render(&self.raw_polygon_style, &context)?;

            for point in self.raw_polygon.exterior_coords_iter() {
                point.render(&self.raw_point_style, &context)?;
            }
        }

        Ok(())
    }
}
