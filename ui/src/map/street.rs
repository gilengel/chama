extern crate rust_editor;

use std::collections::HashMap;

use geo::{
    euclidean_length::EuclideanLength,
    line_intersection::LineIntersection,
    prelude::{Contains, EuclideanDistance, Centroid},
    Coordinate, Line, LineString, Point, Polygon,
};
use rust_editor::{
    gizmo::{GetPosition, Id, SetId},
    interactive_element::{InteractiveElement, InteractiveElementState},
    renderer::PrimitiveRenderer,
    style::{InteractiveElementStyle, Style},
};
use rust_macro::ElementId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use geo::line_intersection::line_intersection;

use super::intersection::{Intersection, Side};

#[derive(Clone, Serialize, Deserialize, ElementId, Debug, PartialEq)]
pub struct Street {
    id: Uuid,

    pub line: Line<f64>,

    polygon: Polygon<f64>,

    width: f64,

    pub start: Uuid,
    pub end: Uuid,

    left_next: Option<Uuid>,
    right_next: Option<Uuid>,

    left_previous: Option<Uuid>,
    right_previous: Option<Uuid>,

    norm: Coordinate<f64>,

    inverse_norm: Coordinate<f64>,

    pub style: InteractiveElementStyle,

    state: InteractiveElementState,
}

impl<'a> From<&'a Street> for &'a Line<f64> {
    fn from(street: &'a Street) -> &'a Line<f64> {
        &street.line
    }
}

impl Default for Street {
    fn default() -> Self {
        Street {
            id: Uuid::new_v4(),
            line: Line::new(Point::new(0.0, 0.0), Point::new(0.0, 0.0)),
            width: 20.0,
            polygon: Polygon::new(LineString::from(vec![Coordinate { x: 0., y: 0. }]), vec![]),
            start: Uuid::default(),
            end: Uuid::default(),

            left_next: None,
            right_next: None,
            left_previous: None,
            right_previous: None,

            norm: Coordinate { x: 0.0, y: 0.0 },
            inverse_norm: Coordinate { x: 0.0, y: 0.0 },

            style: InteractiveElementStyle::default(),
            state: InteractiveElementState::Normal,
        }
    }
}

impl InteractiveElement for Street {
    fn set_state(&mut self, new_state: InteractiveElementState) {
        self.state = new_state;
    }

    fn style(&self) -> &Style {
        match self.state {
            InteractiveElementState::Normal => return &self.style.normal,
            InteractiveElementState::Hover => return &self.style.hover,
            InteractiveElementState::Selected => return &self.style.selected,
        }
    }

    fn state(&self) -> InteractiveElementState {
        InteractiveElementState::Normal
    }
}

impl Street {
    pub fn start(&self) -> Coordinate<f64> {
        self.line.start
    }

    pub fn set_start(&mut self, start: &Intersection) {
        self.start = start.id();
        self.line.start = start.position();
    }

    pub fn set_end(&mut self, end: &Intersection) {
        self.end = end.id();
        self.line.end = end.position();
    }

    pub fn norm(&self) -> Coordinate<f64> {
        self.norm
    }

    pub fn inverse_norm(&self) -> Coordinate<f64> {
        self.inverse_norm
    }

    pub fn end(&self) -> Coordinate<f64> {
        self.line.end
    }

    pub fn set_previous(&mut self, side: Side, id: Option<Uuid>) {
        match side {
            Side::Left => self.left_previous = id,
            Side::Right => self.right_previous = id,
        }
    }

    pub fn get_previous(&self, side: Side) -> Option<Uuid> {
        match side {
            Side::Left => self.left_previous,
            Side::Right => self.right_previous,
        }
    }

    pub fn set_next(&mut self, side: Side, id: Option<Uuid>) {
        match side {
            Side::Left => self.left_next = id,
            Side::Right => self.right_next = id,
        }
    }

    pub fn get_next(&self, side: Side) -> Option<Uuid> {
        match side {
            Side::Left => self.left_next,
            Side::Right => self.right_next,
        }
    }

    pub fn get_side_of_position(&self, position: &Coordinate<f64>) -> Side {
        let start: Point<f64> = self.start().into();

        if start.cross_prod(self.end().into(), (*position).into()) < 0.0 {
            return Side::Left;
        }

        Side::Right
    }

    pub fn length(&self) -> f64 {
        self.line.euclidean_length()
    }

    pub fn polygon(&self) -> &Polygon<f64> {
        &self.polygon
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn update_geometry(
        &mut self,
        intersections: &HashMap<Uuid, Intersection>,
        streets: &HashMap<Uuid, Street>,
    ) {
        if let (Some(start), Some(end)) =
            (intersections.get(&self.start), intersections.get(&self.end))
        {
            self.line.start = start.position();
            self.line.end = end.position();

            let start = self.line.start;
            let end = self.line.end;

            let length = start.euclidean_distance(&end);
            let vec = end - start;
            self.norm = Coordinate {
                x: vec.x / length,
                y: vec.y / length,
            };

            let inverse_vec = start - end;
            self.inverse_norm = Coordinate {
                x: inverse_vec.x / length,
                y: inverse_vec.y / length,
            };

            let pts = self.calc_polygon_points(streets);
            self.polygon = Polygon::new(LineString::from(pts), vec![]);
        }
    }

    pub fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        self.polygon.render(self.style(), &context)?;
        

        Ok(())
    }

    pub fn perp(&self) -> Coordinate<f64> {
        Coordinate {
            x: -self.norm.y,
            y: self.norm.x,
        }
    }

    pub fn are_norms_equal(&self, another_street: &Street) -> bool {
        let pt1 = self.norm();
        let pt2 = another_street.norm();
        let r = pt1 - pt2;
        let r2 = pt1 + pt2;

        if (r.x.abs() < 0.001 && r.y.abs() < 0.001) || (r2.x.abs() < 0.001 && r2.y.abs() < 0.001) {
            return true;
        }

        false
    }

    fn calc_polygon_points(&self, streets: &HashMap<Uuid, Street>) -> Vec<Coordinate<f64>> {
        let half_width = self.width / 2.0;
        let s = self.start();

        let length = self.line.euclidean_length();

        let mut points: Vec<Coordinate<f64>> = vec![];
        let perp = Point::new(-self.norm.y, self.norm.x);
        let offset: Coordinate<f64> = (perp * half_width).into();

        let end = s + (self.norm() * length).into();
        points.push(s.into());
        points.push(s - offset);
        points.push(end - offset);
        points.push(end);
        points.push(end + offset);
        points.push(s + offset);

        if let Some(next_left) = &self.left_next {
            if let Some(next_left) = streets.get(next_left) {
                let factor = if self.end == next_left.end { 1.0 } else { -1.0 };

                let offset = next_left.start() + next_left.perp() * half_width * factor;
                let start = points[1];

                if !self.are_norms_equal(&next_left) {
                    if let Some(intersection) =
                        self.line_intersect_line(start, self.norm, offset, next_left.norm)
                    {
                        points[2] = intersection;
                    }
                }
            }
        }

        if let Some(right_next) = &self.right_next {
            if let Some(right_next) = streets.get(right_next) {
                let factor = if self.end == right_next.end {
                    -1.0
                } else {
                    1.0
                };

                let offset = right_next.start() + right_next.perp() * half_width * factor;
                let start = *points.last().unwrap();

                if !self.are_norms_equal(&right_next) {
                    if let Some(intersection) =
                        self.line_intersect_line(start, self.norm, offset, right_next.norm)
                    {
                        let pts_len = points.len();

                        points[pts_len - 2] = intersection;
                    }
                }
            }
        }

        if let Some(previous_left) = &self.left_previous {
            if let Some(previous_left) = streets.get(previous_left) {
                let factor = if self.start == previous_left.start {
                    1.0
                } else {
                    -1.0
                };

                let start = points[1];
                let other_start =
                    previous_left.start() + previous_left.perp() * half_width * factor;

                if !self.are_norms_equal(&previous_left) {
                    if let Some(intersection) =
                        self.line_intersect_line(start, self.norm, other_start, previous_left.norm)
                    {
                        points[1] = intersection;
                    }
                }
            }
        }

        if let Some(right_previous) = &self.right_previous {
            if let Some(right_previous) = streets.get(right_previous) {
                let factor = if self.start == right_previous.start {
                    -1.0
                } else {
                    1.0
                };

                let offset = right_previous.start() + right_previous.perp() * half_width * factor;
                if !self.are_norms_equal(&right_previous) {
                    if let Some(intersection) = self.line_intersect_line(
                        *points.last().unwrap(),
                        self.norm,
                        offset,
                        right_previous.norm,
                    ) {
                        *points.last_mut().unwrap() = intersection;
                    }
                }
            }
        }

        points
    }

    fn line_intersect_line(
        &self,
        start: Coordinate<f64>,
        start_dir: Coordinate<f64>,
        end: Coordinate<f64>,
        end_dir: Coordinate<f64>,
    ) -> Option<Coordinate<f64>> {
        let line1 = Line::new(start + start_dir * -1000.0, start + start_dir * 1000.0);
        let line2 = Line::new(end + end_dir * -1000.0, end + end_dir * 1000.0);

        if let Some(intersection) = line_intersection(line1, line2) {
            match intersection {
                LineIntersection::SinglePoint {
                    intersection,
                    is_proper: _,
                } => {
                    return Some(intersection);
                }
                _ => {}
            }
        }

        None
    }

    pub fn intersect_with_street(&self, another: &Street) -> Option<LineIntersection<f64>> {
        line_intersection(self.line, another.line)
    }

    pub fn intersect_with_line(&self, another: &Line<f64>) -> Option<LineIntersection<f64>> {
        line_intersection(self.line, *another)
    }

    pub fn is_point_on_street(&self, point: &Coordinate<f64>) -> bool {
        self.polygon.contains(point)
    }
}
