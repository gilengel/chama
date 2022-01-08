use std::{cell::RefCell, rc::Rc};

use geo::{
    line_intersection::LineIntersection,
    prelude::{Contains, EuclideanDistance},
    Coordinate, Line, LineString, Point, Polygon,
};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use geo::line_intersection::line_intersection;

use crate::intersection::{Intersection, Side};

#[derive(Clone)]
pub struct Street {
    pub id: u32,
    pub line: Line<f64>,
    polygon: Polygon<f64>,

    width: f64,

    pub start: Option<Rc<RefCell<Intersection>>>,
    pub end: Option<Rc<RefCell<Intersection>>>,

    left_next: Option<Rc<RefCell<Street>>>,
    right_next: Option<Rc<RefCell<Street>>>,

    left_previous: Option<Rc<RefCell<Street>>>,
    right_previous: Option<Rc<RefCell<Street>>>,

    norm: Point<f64>,
    inverse_norm: Point<f64>,
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

            left_next: None,
            right_next: None,
            left_previous: None,
            right_previous: None,

            norm: Point::new(0.0, 0.0),
            inverse_norm: Point::new(0.0, 0.0),
        }
    }
}

impl Street {
    pub fn start(&self) -> Coordinate<f64> {
        self.start.as_ref().unwrap().borrow().get_position()
    }

    /*
    pub fn id(&self) -> u32 {
        self.id
    }
    */

    pub fn set_start(&mut self, start: Rc<RefCell<Intersection>>) {
        self.start = Some(start);

        self.line.start = self.start();
    }

    pub fn set_start_position(&mut self, pos: &Coordinate<f64>) {
        self.start
            .as_ref()
            .unwrap()
            .borrow_mut()
            .set_position(pos.clone());
    }

    pub fn norm(&self) -> Point<f64> {
        self.norm
    }

    pub fn inverse_norm(&self) -> Point<f64> {
        self.inverse_norm
    }

    pub fn end(&self) -> Coordinate<f64> {
        self.end.as_ref().unwrap().borrow().get_position()
    }

    pub fn set_end(&mut self, end: Rc<RefCell<Intersection>>) {
        self.end = Some(end);
        self.line.end = self.end();

        self.update_geometry();
    }

    pub fn set_end_position(&mut self, pos: &Coordinate<f64>) {
        self.end
            .as_ref()
            .unwrap()
            .borrow_mut()
            .set_position(pos.clone());
    }

    pub fn set_previous(&mut self, side: Side, street: Option<Rc<RefCell<Street>>>) {
        match side {
            Side::Left => self.left_previous = street,
            Side::Right => self.right_previous = street,
        }   
    }

    pub fn get_previous(&self, side: Side) -> Option<&Rc<RefCell<Street>>> {
        match side {
            Side::Left => self.left_previous.as_ref(),
            Side::Right => self.right_previous.as_ref(),
        }
    }

    pub fn set_next(&mut self, side: Side, street: Option<Rc<RefCell<Street>>>) {
        match side {
            Side::Left => self.left_next = street,
            Side::Right => self.right_next = street,
        }   
    }

    pub fn get_next(&self, side: Side) -> Option<&Rc<RefCell<Street>>> {
        match side {
            Side::Left => self.left_next.as_ref(),
            Side::Right => self.right_next.as_ref(),
        }
    }

    pub fn get_side_of_position(&self, position: &Coordinate<f64>) -> Side {
        let start : Point<f64> = self.start().into();

        if start.cross_prod(self.end().into(), (*position).into()) < 0.0 {
            return Side::Left;
        }
        
        Side::Right
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

        let inverse_vec = self.start() - self.end();
        self.inverse_norm = Point::new(inverse_vec.x / length, inverse_vec.y / length);

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
        context.set_fill_style(&"#2A2A2B".into());
        context.fill();

        /*
        let mut owned_string: String = format!("{} -> ", self.id);


        match &self.left_previous {
            Some(l) => {
                owned_string.push_str(format!("{},", l.as_ref().borrow().id()).as_str())
            },
            None => owned_string.push_str("#,"),
        }
        match &self.right_previous {
            Some(l) => {
                owned_string.push_str(format!("{},", l.as_ref().borrow().id()).as_str())
            },
            None => owned_string.push_str("#,"),
        }
        match &self.left_next {
            Some(l) => {
                owned_string.push_str(format!("{},", l.as_ref().borrow().id()).as_str())
            },
            None => owned_string.push_str("#,"),
        }
        match &self.right_next {
            Some(l) => {
                owned_string.push_str(format!("{}", l.as_ref().borrow().id()).as_str())
            },
            None => owned_string.push_str("#"),
        }

        if let Some(position) = self.polygon.exterior().centroid() {
            context.set_fill_style(&"#FFFFFF".into());
            context.fill_text(
                &owned_string,
                position.x(),
                position.y(),
            )?;
        }
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
