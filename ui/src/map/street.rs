extern crate rust_editor;


use geo::{
    line_intersection::LineIntersection,
    prelude::{Contains, EuclideanDistance},
    Coordinate, Line, LineString, Polygon,
};
use rust_editor::{
    gizmo::{Id, SetId},
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

#[derive(Clone, Serialize, Deserialize, ElementId, Debug, PartialEq)]
pub struct Street {
    id: Uuid,

    pub lines: LineString<f64>,

    polygon: Polygon<f64>,

    width: f64,

    norm: Coordinate<f64>,

    inverse_norm: Coordinate<f64>,

    pub style: InteractiveElementStyle,

    state: InteractiveElementState,
}

trait Norm {
    fn norm(&self) -> Coordinate<f64>;
}

impl Norm for Line<f64> {
    fn norm(&self) -> Coordinate<f64> {
        let start = self.start;
        let end = self.end;

        let vec = end - start;

        let length = start.euclidean_distance(&end);

        Coordinate {
            x: vec.x / length,
            y: vec.y / length,
        }
    }
}

struct EnhancedLine {
    line: Line<f64>,
    norm: Coordinate<f64>,
    perp: Coordinate<f64>,
}

impl EnhancedLine {
    fn new(line: Line<f64>) -> Self {
        let start = line.start;
        let end = line.end;

        let length = start.euclidean_distance(&end);
        let vec = end - start;
        let norm = Coordinate {
            x: vec.x / length,
            y: vec.y / length,
        };

        let perp = Coordinate {
            x: -norm.y,
            y: norm.x,
        };

        EnhancedLine { line, norm, perp }
    }

    fn start(&self) -> Coordinate<f64> {
        self.line.start
    }

    fn end(&self) -> Coordinate<f64> {
        self.line.end
    }
}

fn line_intersect_line(
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

fn calc_polygon_points<I>(it: I, width: f64) -> Polygon<f64>
where
    I: Iterator<Item = Line<f64>>,
{
    let streets = it
        .map(|line| EnhancedLine::new(line))
        .collect::<Vec<EnhancedLine>>();
    let mut it = streets.iter().peekable();

    let mut points: Vec<Coordinate<f64>> = vec![]; //Vec::with_capacity(num_pts * 2 + 2);
    let mut points2: Vec<Coordinate<f64>> = vec![];

    let half_width = width / 2.0;
    let s = it.next().unwrap();

    points.push(s.start() + s.perp * half_width);
    points2.push(s.start() + s.perp * -half_width);

    while it.peek().is_some() {
        let current_line = it.next().unwrap();

        let offset = current_line.perp * half_width;
        let next_pt = match it.peek() {
            Some(next_line) => {
                let next_offset = next_line.perp * half_width;

                (
                    line_intersect_line(
                        current_line.end() + offset,
                        current_line.norm,
                        next_line.start() + next_offset,
                        next_line.norm,
                    )
                    .unwrap_or_else(|| current_line.end() + offset),
                    line_intersect_line(
                        current_line.end() + offset * -1.,
                        current_line.norm,
                        next_line.start() + next_offset * -1.,
                        next_line.norm,
                    )
                    .unwrap_or_else(|| current_line.end() + offset * -1.),
                )
            }
            None => {
                let pt = current_line.end();

                (pt + offset, pt + offset * -1.)
            }
        };

        points.push(next_pt.0);
        points2.push(next_pt.1);
    }

    points2.reverse();
    points.append(&mut points2);

    Polygon::new(LineString::new(points), vec![])
}

impl Default for Street {
    fn default() -> Self {
        Street {
            id: Uuid::new_v4(),
            width: 20.0,
            polygon: Polygon::new(LineString::from(vec![Coordinate { x: 0., y: 0. }]), vec![]),
            lines: LineString::new(vec![]),

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
    pub fn new(line_string: LineString<f64>) -> Self {
        let it = line_string.lines().into_iter().peekable();
        let polygon = calc_polygon_points(it, 20.);

        Street {
            id: Uuid::new_v4(),
            lines: line_string,
            polygon,
            ..Default::default()
        }
    }

    pub fn norm(&self) -> Coordinate<f64> {
        self.norm
    }

    pub fn inverse_norm(&self) -> Coordinate<f64> {
        self.inverse_norm
    }

    pub fn polygon(&self) -> &Polygon<f64> {
        &self.polygon
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        self.polygon.render(self.style(), context)?;

        /*
                context.set_fill_style(&"#FFFFFF".into());

                for it in self.lines.coords_iter() {
                    context.fill_text(
                        &format!(
                            "{:?}",
                            it.x_y()
                        )
                        .to_string(),
                        it.x,
                        it.y,
                    )?;
                }

        */

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

    pub fn is_point_on_street(&self, point: &Coordinate<f64>) -> bool {
        self.polygon.contains(point)
    }
}
