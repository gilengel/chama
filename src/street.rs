use std::{rc::Rc, cell::RefCell, f64::consts::PI};

use geo::{Line, Polygon, LineString, Point, Coordinate, prelude::{Contains, Centroid, EuclideanDistance}, line_intersection::LineIntersection};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use geo::line_intersection::line_intersection;

use crate::intersection::Intersection;

#[derive(Clone)]
pub struct Street {
    pub id: u32,
    pub line: Line<f64>,
    polygon: Polygon<f64>,

    width: f64,

    pub start: Option<Rc<RefCell<Intersection>>>,
    pub end: Option<Rc<RefCell<Intersection>>>,

    norm: Point<f64>,
}

impl Default for Street {
    fn default() -> Self {
        Street {
            id: u32::MAX,
            line: Line::new(Point::new(0.0, 0.0), Point::new(0.0, 0.0)),
            width: 20.0,
            polygon: Polygon::new(LineString::from(vec![Coordinate { x: 0., y: 0. }]), vec![]),
            start: None,
            end: None,

            norm: Point::new(0.0, 0.0)
        }
    }
}

impl Street {
    pub fn start(&self) -> Coordinate<f64> {
        self.start.as_ref().unwrap().borrow().get_position()
    }

    pub fn set_start(&mut self, start: Rc<RefCell<Intersection>>) {
        self.start = Some(start);

        self.line.start = self.start();
    }

    pub fn norm(&self) -> Point<f64> {
        self.norm
    }

    pub fn end(&self) -> Coordinate<f64> {
        self.end.as_ref().unwrap().borrow().get_position()
    }

    pub fn set_end(&mut self, end: Rc<RefCell<Intersection>>) {
        self.end = Some(end);
        self.line.end = self.end();

        self.update_geometry();
    }

    pub fn update_geometry(&mut self) {
        let half_width = self.width / 2.0;
        let start: Point<f64> = self.start().into();
        let end: Point<f64> = self.end().into();

        self.line.start = start.into();
        self.line.end = end.into();

        let length = start.euclidean_distance(&end);
        let vec = self.end() - self.start();
        self.norm = Point::new(vec.x / length, vec.y / length);
        let perp = Point::new(-self.norm.y(), self.norm.x());
        let offset = perp * half_width;

        self.polygon = Polygon::new(
            LineString::from(vec![
                start - offset,
                start + self.norm * length - offset,
                start + self.norm * length + offset,
                start + offset,
            ]),
            vec![],
        );
    }

    pub fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        let mut it = self.polygon.exterior().points_iter();
        let start = it.next().unwrap();

        context.begin_path();
        context.move_to(start.x(), start.y());
        for point in it {
            context.line_to(point.x(), point.y());
        }  

        context.close_path();
        //context.fill();
        context.stroke();

        let center = self.polygon.centroid().unwrap();
        context.fill_text(
            &format!("{}", self.id).to_string(),
            center.x(),
            center.y(),
        )?;  

        /*
        let pos = self.line.start_point() + self.norm * self.line.start_point().euclidean_distance(&self.line.end_point()) * 0.8;
        context.begin_path();
        context.arc(pos.x(), pos.y(), 20.0, 0.0, 2.0 * PI)?;
        context.set_fill_style(&"#FF8CFF".into());
        context.fill();  
        */      

        /*
        context.begin_path();
        context.move_to(self.line.start.x, self.line.start.y);
        context.line_to(self.line.end.x, self.line.end.y);
        context.stroke();
        context.close_path();
        */
        Ok(())
    }

    pub fn intersect_with_street(&self, another: &Street) -> Option<LineIntersection<f64>> {
        line_intersection(self.line, another.line)
    }

    pub fn is_point_on_street(&self, point: &Coordinate<f64>) -> bool {
        self.polygon.contains(point)
    }
}