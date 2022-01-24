use std::{
    cmp::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

use geo::{prelude::EuclideanDistance, simplify::Simplify, Coordinate, LineString};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{
    intersection::Intersection, log, map::InformationLayer, renderer::apply_style, state::State,
    street::Street, style::Style, Camera, Map, Renderer, gizmo::SetPosition,
};

pub struct CreateFreeFormStreetState {
    raw_points: Vec<Coordinate<f64>>,
    raw_point_style: Style,

    brush_active: bool,
}

impl Default for CreateFreeFormStreetState {
    fn default() -> Self {
        CreateFreeFormStreetState {
            raw_points: Vec::new(),
            raw_point_style: Style {
                border_width: 15,
                border_color: "#2A2A2B".to_string(),
                background_color: "#2A2A2B".to_string(),
            },
            brush_active: false,
        }
    }
}

impl CreateFreeFormStreetState {
    pub fn new() -> CreateFreeFormStreetState {
        CreateFreeFormStreetState::default()
    }

    fn create_intersection(&self, pos: &Coordinate<f64>) -> Intersection {
        let mut intersection = Intersection::default();
        intersection.set_position(*pos);

        intersection
    }

    pub fn transform_polygon_into_streets(&self, map: &mut Map) {
        let _intersections: Vec<Uuid> = vec![];

        let mut index_to_be_skipped = 0;
        for (index, point) in self.raw_points.iter().enumerate() {
            if map.get_street_at_position(point, &vec![]).is_none() && index != 0 {
                index_to_be_skipped = index - 1;
                break;
            }
        }

        for (i, _) in self.raw_points.iter().enumerate().skip(1) {
            log!(
                "{}->{} = {}",
                i - 1,
                i,
                self.raw_points[i - 1].euclidean_distance(&self.raw_points[i])
            );
        }

        let mut previous = &self.raw_points[index_to_be_skipped];
        for point in self.raw_points.iter().skip(index_to_be_skipped + 1) {
            map.create_street(&previous, point, 10.0);

            previous = point;
        }
    }
}

impl State for CreateFreeFormStreetState {
    fn mouse_down(&mut self, _: Coordinate<f64>, button: u32, _: &mut Map) {
        if button == 0 {
            self.brush_active = true;
        }

        log!("{}", button);
    }

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, _: &mut Map) {
        if self.brush_active {
            self.raw_points.push(mouse_pos);
        }
    }

    fn mouse_up(&mut self, _: Coordinate<f64>, button: u32, map: &mut Map) {
        if button == 0 {
            self.brush_active = false;
        }

        let line_string = LineString(self.raw_points.clone());
        let points = line_string.simplify(&4.0).into_points();
        self.raw_points = points
            .iter()
            .map(|x| Coordinate { x: x.x(), y: x.y() })
            .collect();
        self.transform_polygon_into_streets(map);
        self.raw_points.clear();
    }

    fn render(
        &self,
        map: &Map,
        context: &CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
        camera: &Camera,
    ) -> Result<(), JsValue> {
        map.render(context, additional_information_layer, camera)?;

        if self.brush_active && !self.raw_points.is_empty() {
            context.begin_path();
            context.move_to(self.raw_points[0].x + camera.x as f64, self.raw_points[0].y + camera.y as f64);

            for point in self.raw_points.iter().skip(1) {
                context.line_to(point.x + camera.x as f64, point.y + camera.y as f64);
                context.stroke();
                context.move_to(point.x + camera.x as f64, point.y + camera.y as f64)
            }
            //context.close_path();
            apply_style(&self.raw_point_style, &context);
        }

        Ok(())
    }

    fn enter(&self, _: &mut Map) {}

    fn exit(&self, _: &mut Map) {}
}
