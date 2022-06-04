use core::num;
use std::f64::consts::PI;

use geo::{
    euclidean_length::EuclideanLength,
    line_intersection::{line_intersection, LineIntersection},
    Coordinate, Line, LineString, Point, Polygon, Rect, prelude::{ConvexHull, BoundingRect, Centroid, Area}, rotate::RotatePoint,
};
use rust_editor::log;
type AnnotatedLine = (Line<f64>, bool);

impl AnnotatedPolygon {
    pub(crate) fn lines(&self) -> Vec<AnnotatedLine> {
        self.0
            .exterior()
            .lines()
            .zip(self.1.clone().into_iter())
            .collect()
    }

    pub(crate) fn enclosed(&self) -> bool {
        self.1.iter().all(|is_street| !*is_street)
    }
}

#[derive(Clone)]
pub struct AnnotatedPolygon(pub Polygon<f64>, pub Vec<bool>);

#[derive(Clone, Copy)]
struct PolygonLineIntersection {
    line_segment_index: usize,
    intersection: Coordinate<f64>,
}

fn intersections(
    intersecting_line: &Line<f64>,
    lines: &Vec<AnnotatedLine>,
) -> Vec<PolygonLineIntersection> {
    let mut intersections: Vec<PolygonLineIntersection> = Vec::new();
    for (line_segment_index, segment) in lines.iter().enumerate() {
        match line_intersection(*intersecting_line, segment.0) {
            None => continue,
            Some(line_intersection) => match line_intersection {
                LineIntersection::SinglePoint {
                    intersection,
                    is_proper: _,
                } => intersections.push(PolygonLineIntersection {
                    line_segment_index,
                    intersection,
                }),
                LineIntersection::Collinear { intersection: _ } => continue,
            },
        }
    }

    let num_intersections = intersections.len();
    if num_intersections > 2 {
        intersections.drain(0..2);
    }

    intersections
}

fn calc_intersection_pairs(
    intersections: &Vec<PolygonLineIntersection>,
) -> Vec<(PolygonLineIntersection, PolygonLineIntersection)> {
    let mut pairs: Vec<(PolygonLineIntersection, PolygonLineIntersection)> = Vec::new();

    let mut it = intersections.iter();
    while let Some(line_intersection) = it.next() {
        match it.next() {
            Some(x) => pairs.push((*line_intersection, *x)),
            None => {}
        };
    }

    pairs
}

#[derive(PartialEq)]
struct Crossback<'a> {
    pub lines: Vec<(Line<f64>, &'a bool)>,
    pub crossback: Option<usize>,
}


pub fn longest_and_shortest_diameter(polygon: &Polygon<f64>) -> (f64, f64) {
    let convex_hull = polygon.convex_hull();

    let mut min_area = f64::MAX;
    let mut min_angle = 0.;

    let mut min_poly: Polygon<f64> = convex_hull.bounding_rect().unwrap().into();

    let mut pair = (f64::MAX, f64::MAX);
    for line in convex_hull.exterior().lines() {

        let angle = (-1. * f64::atan2(line.end.y - line.start.y, line.end.x - line.start.x)) * 180.0 / PI;

        
        let mut bbox: Polygon<f64> = convex_hull.bounding_rect().unwrap().into();
        bbox = convex_hull.rotate_around_point(angle, bbox.centroid().unwrap()).bounding_rect().unwrap().into();
        let area = bbox.signed_area();

        if area < min_area {
            min_area = area;
            min_angle = angle;
            min_poly = bbox.clone();

            bbox = bbox.rotate_around_point(-angle, bbox.centroid().unwrap());
            let bbox = bbox.bounding_rect().unwrap();
            pair = (bbox.width(), bbox.height());
        }       
    }

    pair
}


pub fn split(polygon: &AnnotatedPolygon, line: &Line<f64>) -> Vec<AnnotatedPolygon> {
    let intersections = intersections(line, &polygon.lines());

    calc_split_polygons(polygon, &intersections)
}

fn calc_split_polygons(
    polygon: &AnnotatedPolygon,
    intersections: &Vec<PolygonLineIntersection>,
) -> Vec<AnnotatedPolygon> {
    let mut result: Vec<Crossback> = Vec::new();

    let intersection_pairs = calc_intersection_pairs(intersections);
    let mut current_index = 0;

    result.push(Crossback {
        lines: Vec::new(),
        crossback: None,
    });

    let lines = polygon.lines();
    for (i, (line, is_street)) in lines.iter().enumerate() {
        match intersections.iter().find(|x| x.line_segment_index == i) {
            Some(intersection) => {
                let mut other_point_index: Option<usize> = None;
                for k in intersection_pairs.iter() {
                    if intersection.line_segment_index == k.0.line_segment_index {
                        other_point_index = Some(k.1.line_segment_index);
                    }

                    if intersection.line_segment_index == k.1.line_segment_index {
                        other_point_index = Some(k.0.line_segment_index);
                    }

                    let start = if intersection.line_segment_index == 0 {
                        line.start
                    } else {
                        result[current_index].lines.last().unwrap().0.end
                    };

                    result[current_index].lines.push((
                        Line::new(start, intersection.intersection),
                        &lines[intersection.line_segment_index].1,
                    ));

                    result[current_index].crossback = other_point_index;
                }

                match result
                    .iter()
                    .filter(|x| **x != result[current_index])
                    .position(|x| {
                        x.crossback.is_some()
                            && x.crossback.unwrap() == intersection.line_segment_index
                    }) {
                    Some(k) => {
                        let line = Line::new(
                            result[current_index].lines.last().unwrap().0.end,
                            result[k].lines.first().unwrap().0.start,
                        );

                        result[current_index].lines.push((line, &false));

                        let opposide_line = Line::new(
                            result[k].lines.last().unwrap().0.end,
                            result[current_index].lines.last().unwrap().0.end,
                        );

                        current_index = k;
                        result[current_index].lines.push((opposide_line, &false));
                    }
                    None => {
                        result.push(Crossback {
                            lines: Vec::new(),
                            crossback: None,
                        });
                        current_index = result.len() - 1;
                    }
                }

                result[current_index].lines.push((
                    Line::new(intersection.intersection, line.end),
                    &lines[intersection.line_segment_index].1,
                ));
            }
            None => result[current_index].lines.push((line.clone(), is_street)),
        }
    }

    result
        .iter()
        .map(|x| {
            let pts: Vec<Point<f64>> = x.lines.iter().map(|x| x.0.start_point()).collect();
            let poly = Polygon::new(LineString::from(pts), vec![]);
            let is_street: Vec<bool> = x.lines.iter().map(|(_, x)| **x).collect();

            AnnotatedPolygon(poly, is_street)
        })
        .collect()
}

pub fn longest_line(polygon: &AnnotatedPolygon, min_side_length: f64) -> AnnotatedLine {
    fn determine_longest_line<'a, It>(it: It) -> Option<&'a AnnotatedLine>
    where
        It: Iterator<Item = &'a (Line<f64>, bool)>,
    {
        it.max_by(|x, y| {
            x.0.euclidean_length()
                .partial_cmp(&y.0.euclidean_length())
                .unwrap()
        })
    }

    match determine_longest_line(
        polygon
            .lines()
            .iter()
            .filter(|(line, is_street)| *is_street && line.euclidean_length() >= min_side_length),
    ) {
        Some(line) => *line,
        None => *determine_longest_line(polygon.lines().iter()).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use geo::{Coordinate, Line, LineString, Point, Polygon};

    use crate::algorithm::geo::longest_line;

    use super::{split, AnnotatedPolygon};

    #[test]
    fn unit_split_polygon() {
        let pts: Vec<Point<f64>> = vec![
            Point::new(0., 0.),
            Point::new(256., 0.),
            Point::new(256., 256.),
            Point::new(0., 256.),
        ];
        let poly = Polygon::new(LineString::from(pts.clone()), vec![]);
        let is_street: Vec<bool> = pts.iter().map(|_| true).collect();

        let polygon = AnnotatedPolygon(poly, is_street);

        let splits = split(
            &polygon,
            &Line::new(
                Coordinate { x: 128., y: 0. },
                Coordinate { x: 128., y: 256. },
            ),
        );

        assert_eq!(splits.len(), 2);
        assert_eq!(splits[0].enclosed(), false);
        assert_eq!(splits[1].enclosed(), false);
    }

    #[test]
    fn unit_longest_line() {
        let pts: Vec<Point<f64>> = vec![
            Point::new(0., 0.),
            Point::new(256., 0.),
            Point::new(256., 512.),
            Point::new(0., 256.),
        ];
        let poly = Polygon::new(LineString::from(pts.clone()), vec![]);
        let is_street: Vec<bool> = pts.iter().map(|_| true).collect();

        let polygon = AnnotatedPolygon(poly, is_street);

        let result = longest_line(&polygon, 5.0);

        assert_eq!(
            result.0,
            Line::new(
                Coordinate { x: 256., y: 0. },
                Coordinate { x: 256., y: 512. }
            )
        );
        assert_eq!(result.1, true);
    }
}
