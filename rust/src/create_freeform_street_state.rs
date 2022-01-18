use geo::{Coordinate, Polygon, LineString, Point, simplify::Simplify};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{state::{State}, Map, map::InformationLayer, Camera, Renderer, style::Style, intersection::Intersection, street::Street, district::create_district_for_street, renderer::{PrimitiveRenderer, apply_style}};


pub struct CreateFreeFormStreetState {
    raw_points: Vec<Coordinate<f64>>,
    raw_point_style: Style,

    brush_active: bool
}

impl Default for CreateFreeFormStreetState {
    fn default() -> Self {
        CreateFreeFormStreetState {
            raw_points: Vec::new(),
            raw_point_style: Style {
                border_width: 15,
                border_color: "#2A2A2B".to_string(),
                background_color: "#2A2A2B".to_string()
            },
            brush_active: false
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
        let mut intersections: Vec<Uuid> = vec![];

        for point in &self.raw_points {
            intersections.push(map.add_intersection(self.create_intersection(&point)));
        }

        let mut street_id = Uuid::default();
        let mut it = intersections.iter().peekable();
        while let Some(current_id) = it.next() {
            if let Some(next_id) = it.peek() {
                let current_intersection = map.intersection(current_id).unwrap();
                let next_intersection = map.intersection(next_id).unwrap();

                let mut street = Street::default();
                street_id = street.id;
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
    }
}


impl State for CreateFreeFormStreetState {
    fn mouse_down(&mut self, _: Coordinate<f64>, button: u32, _: &mut Map) {
        if button == 0 {
            self.brush_active = true;
        }
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
        let points = line_string.simplify(&8.0).into_points();
        self.raw_points = points.iter().map(|x| Coordinate { x: x.x(), y: x.y() }).collect();
        self.transform_polygon_into_streets(map);
        self.raw_points.clear();
    }

    fn render(&self, map: &Map, context: &CanvasRenderingContext2d, additional_information_layer: &Vec<InformationLayer>, camera: &Camera) -> Result<(), JsValue> {
        map.render(context, additional_information_layer, camera)?;

        if self.brush_active && !self.raw_points.is_empty(){
            context.begin_path();
            context.move_to(self.raw_points[0].x, self.raw_points[0].y);

            for point in self.raw_points.iter().skip(1) {
                context.line_to(point.x, point.y);
                context.stroke();
                context.move_to(point.x, point.y)
            }
            //context.close_path();
            apply_style(&self.raw_point_style, &context);
            
        }


        Ok(())
    }

    fn enter(&self, _: &mut Map) { }

    fn exit(&self, _: &mut Map) {}
}