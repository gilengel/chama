use std::cmp::Ordering;

use geo::line_intersection::LineIntersection;
use geo::line_intersection::line_intersection;
use geo::prelude::EuclideanDistance;
use geo_types::Coordinate;
use geo_types::Line;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use geo::euclidean_distance;

use geo_types::Point;

use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

#[derive(Clone)]
struct Street {
    line: Line<f64>,

    width: f64,
}


impl Street {
    pub fn start(&self) -> Coordinate<f64> {
        self.line.start
    }

    pub fn set_start(&mut self, start: Coordinate<f64>) {
        self.line.start = start;
    }

    pub fn end(&self) -> Coordinate<f64> {
        self.line.end
    }

    pub fn set_end(&mut self, end: Coordinate<f64>) {
        self.line.end = end
    }

    pub fn render(&self, context: &CanvasRenderingContext2d) {
        context.begin_path();
        context.move_to(self.start().x, self.start().y);
        context.line_to(self.end().x, self.end().y);
        context.set_line_width(self.width);
        context.stroke();
    }

    pub fn intersect_with_street(&self, another: &Street) -> Option<LineIntersection<f64>> {
        line_intersection(self.line, another.line)
    }
}

impl Default for Street {
    fn default() -> Self {
        Street {
            line: Line::new(Point::new(0.0, 0.0), Point::new(0.0, 0.0)),
            width: 20.0
        }
    }
}

#[wasm_bindgen]
pub struct Editor {
    width: u32,
    height: u32,
    streets: Vec<Street>,

    temp_street: Street,

    context: CanvasRenderingContext2d,

    mouse_pressed: bool,
}

fn get_canvas_and_context() -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("map_canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    Ok((canvas, context))
}

#[wasm_bindgen]
impl Editor {
    pub fn new() -> Editor {
        let (_, context) = get_canvas_and_context().unwrap();
        Editor {
            width: 1920,
            height: 800,
            streets: vec![],
            temp_street: Street::default(),
            context,
            mouse_pressed: false,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn render(&self) {
        self.context
            .clear_rect(0.0, 0.0, self.width.into(), self.height.into());

        self.temp_street.render(&self.context);

        for street in &self.streets {
            street.render(&self.context);
        }
    }

    pub fn mouse_down(&mut self, x: u32, y: u32) {
        self.temp_street.set_start(Coordinate { x: x.into(), y: y.into() });
        self.temp_street.set_end(Coordinate { x: x.into(), y: y.into() });

        self.mouse_pressed = true
    }

    pub fn mouse_up(&mut self, x: u32, y: u32) {
        if self.mouse_pressed {
            //self.temp_street.set_end(Coordinate { x: x.into(), y: y.into() });

            self.streets.push(self.temp_street.clone());

            self.temp_street = Street::default();
        }

        self.mouse_pressed = false;
    }

    pub fn mouse_move(&mut self, x: u32, y: u32) {
        if self.mouse_pressed {
            self.temp_street.set_end(Coordinate { x: x.into(), y: y.into() });

            if let Some(intersection) = self.intersect_with_streets() {
                self.temp_street.set_end(intersection)
            }
        }
    }

    fn intersect_with_streets(&self) -> Option<Coordinate<f64>> {
        let mut intersections = vec![];

        for street in &self.streets {            
            if let Some(line_intersection) = self.temp_street.intersect_with_street(street) {
                match line_intersection {
                    LineIntersection::SinglePoint { intersection, is_proper } => 
                    {
                        if is_proper {
                            intersections.push(intersection);
                        }
                    },
                    _ => {},
                }
            }
        }

        intersections.sort_by(|a, b| {
            let d1 = a.euclidean_distance(&self.temp_street.start());
            let d2 = b.euclidean_distance(&self.temp_street.start());

            if d1 < d2 {
                return Ordering::Less
            }
            
            if d1 == d2 {
                return Ordering::Equal
            }

            Ordering::Greater            
        });

        if intersections.is_empty() {
            return None
        }
        
        Some(intersections.first().unwrap().clone())
    }
}
