use std::iter::Peekable;

use geo::{
    dimensions::HasDimensions,
    line_intersection::{line_intersection, LineIntersection},
    prelude::{EuclideanDistance, Area},
    winding_order::Winding,
    Coordinate, Line, LineString, Point, Polygon,
};
use rust_editor::log;

use super::district::District;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineSegmentType {
    Street,
    Backside,
}

#[derive(Clone, Copy, Debug)]
pub struct LineSegment {
    pub start: Point<f64>,
    pub end: Point<f64>,
    pub norm: Point<f64>,
    pub perp: Point<f64>,
    pub length: f64,
    pub ty: LineSegmentType,
}

impl LineSegment {
    pub fn new(start: Point<f64>, end: Point<f64>, ty: LineSegmentType) -> LineSegment {
        let length = start.euclidean_distance(&end);
        let vec = end - start;
        let norm = Point::new(vec.x() / length, vec.y() / length);
        let perp = Point::new(-norm.y(), norm.x());

        LineSegment {
            start: start,
            end: end,
            norm,
            perp,
            length,
            ty,
        }
    }

    fn split_point(&self) -> Point<f64> {
        self.start + self.norm * self.length * 0.5
    }
}

impl Into<Line<f64>> for &LineSegment {
    fn into(self) -> Line<f64> {
        Line::new(self.start, self.end)
    }
}

impl PartialEq for LineSegment {
    fn eq(&self, other: &LineSegment) -> bool {
        self.start == other.start && self.end == other.end
    }
}

/*
fn longest_segment(segments: &Vec<LineSegment>) -> &LineSegment {
    segments
        .into_iter()
        .max_by(|x, y| x.length.partial_cmp(&y.length).unwrap())
        .unwrap()
}
*/
fn longest_segment(segments: &Vec<LineSegment>) -> &LineSegment {
    let adjacent_street_segments = segments
        .iter()
        .filter(|segment| segment.ty == LineSegmentType::Street);

    match adjacent_street_segments.max_by(|x, y| x.length.partial_cmp(&y.length).unwrap()) {
        Some(e) => e,
        None => {
            // Corner case for polygons without adjacency to any street
            let e = segments
                .into_iter()
                .max_by(|x, y| x.length.partial_cmp(&y.length).unwrap())
                .unwrap();

            e
        }
    }
}

#[derive(Clone, Copy)]
struct PolygonLineIntersection {
    line_segment_index: usize,
    intersection: Coordinate<f64>,
}

fn intersections(
    longest_segment: &LineSegment,
    segments: &Vec<LineSegment>,
) -> Vec<PolygonLineIntersection> {
    let split_point = longest_segment.split_point();

    let mut intersections: Vec<PolygonLineIntersection> = Vec::new();

    let line = Line::new(
        split_point + longest_segment.perp * 6000.0,
        split_point + longest_segment.perp * -6000.0,
    );

    for (line_segment_index, segment) in segments.iter().enumerate() {
        match line_intersection(line, segment.into()) {
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

    intersections
}

fn calc_intersection_pairs(
    intersections: &Vec<PolygonLineIntersection>,
) -> Vec<(PolygonLineIntersection, PolygonLineIntersection)> {
    let mut pairs: Vec<(PolygonLineIntersection, PolygonLineIntersection)> = Vec::new();

    let mut it = intersections.iter().peekable();
    while let Some(line_intersection) = it.next() {
        match it.peek() {
            Some(x) => pairs.push((*line_intersection, **x)),
            None => {}
        };
    }

    pairs
}

struct Crossback {
    pub points: Vec<(Point<f64>, PointType)>,
    pub crossback: Option<usize>,
}

fn calc_split_polygons(
    polygon: &AnnotatedPolygon,
    intersections: &Vec<PolygonLineIntersection>,
) -> Vec<AnnotatedPolygon> {
    let mut result: Vec<Crossback> = Vec::new();

    let intersection_pairs = calc_intersection_pairs(intersections);

    let mut current_index = 0;

    result.push(Crossback {
        points: Vec::new(),
        crossback: None,
    });

    // Iterate over the polygon in clock wise order
    for (i, (point, adjacent_to_street)) in polygon.iter().enumerate() {
        // Normal case, current segment was not intersected and we only need to add the next point to the current polygon
        result[current_index]
            .points
            .push((*point, *adjacent_to_street));

        // Special case, the current segment index matches an intersection therefore we need to jump to the crossback of it
        if let Some(intersection) = intersections.iter().find(|x| x.line_segment_index == i) {
            let mut other_point_index: Option<usize> = None;

            for k in intersection_pairs.iter() {
                if intersection.line_segment_index == k.0.line_segment_index {
                    other_point_index = Some(k.1.line_segment_index);
                }

                if intersection.line_segment_index == k.1.line_segment_index {
                    other_point_index = Some(k.0.line_segment_index);
                }

                result[current_index]
                    .points
                    .push((intersection.intersection.into(), PointType::Intersection));
                result[current_index].crossback = other_point_index;

                match result.iter().position(|x| {
                    x.crossback.is_some() && x.crossback.unwrap() == intersection.line_segment_index
                }) {
                    Some(k) => {
                        if k == current_index {
                            continue;
                        }

                        if result[k].crossback.unwrap() == intersection.line_segment_index {
                            current_index = k;
                        }
                    }
                    None => {
                        result.push(Crossback {
                            points: Vec::new(),
                            crossback: None,
                        });
                        current_index = result.len() - 1;
                    }
                }

                result[current_index]
                    .points
                    .push((intersection.intersection.into(), PointType::Intersection));
            }
        }
    }

    result
        .iter()
        .map(|x| {
            let mut pts = x.points.clone();
            pts.push(*pts.first().unwrap());

            //let mut adjacencies : Vec<bool> = x.points.iter().map(|(_, x)| *x).collect();
            //adjacencies.push(*adjacencies.first().unwrap());

            //let polygon = Polygon::new(LineString::from(pts.clone()), vec![]);
            //log!("{:?}", adjacencies.contains(&false));
            //log!("B => {:?}", polygon.exterior().points_cw().collect::<Vec<Point<f64>>>());
            //log!("=====================");
            AnnotatedPolygon(pts)
        }) //Polygon::new(LineString::from(x.points.clone()), vec![]))
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
pub enum PointType {
    Street,
    Intersection,
}
#[derive(Clone)]
pub struct AnnotatedPolygon(Vec<(Point<f64>, PointType)>);



impl AnnotatedPolygon {
    pub fn new_adjacent_to_streets(polygon: &Polygon<f64>) -> Self {
        let points: Vec<Point<f64>> = polygon.exterior().points_cw().collect();
        AnnotatedPolygon(points.iter().map(|p| (*p, PointType::Street)).collect())
    }

    pub fn iter(&self) -> Peekable<std::slice::Iter<'_, (geo::Point<f64>, PointType)>> {
        self.0.iter().peekable()
    }

    pub fn size(&self) -> f64 {
        let points : Vec<Point<f64>> = self.0.iter().map(|(x, _)| *x).collect();
        let polygon = Polygon::new(LineString::from(points), vec![]);
        polygon.unsigned_area()
    }
}

fn foo(cnt: u32, polygon: &AnnotatedPolygon, min_side_length: f64) -> Vec<Polygon<f64>> {
    let mut polygons: Vec<Polygon<f64>> = Vec::new();

    //log!("{}", polygon.size());
    if polygon.size() < min_side_length * min_side_length {
        let pts: Vec<Point<f64>> = polygon.iter().map(|(pt, _)| *pt).collect();
        let polygon = Polygon::new(LineString::from(pts), vec![]);
        polygons.push(polygon);
        return polygons;
    }
    if cnt == 10 {

    }

    let mut segments: Vec<LineSegment> = vec![];
    let mut it = polygon.iter();
    while let Some((start_pt, start_pt_type)) = it.next() {
        if let Some((next_pt, end_pt_type)) = it.peek() {
            segments.push(LineSegment::new(
                *start_pt,
                *next_pt,
                if *start_pt_type == PointType::Intersection
                    && *end_pt_type == PointType::Intersection
                {
                    LineSegmentType::Backside
                } else {
                    LineSegmentType::Street
                },
            ));
        }
    }

    let longest_segment = longest_segment(&segments);
    let intersections = intersections(longest_segment, &segments);

    for sub_polygon in calc_split_polygons(&polygon, &intersections) {
        polygons.append(&mut foo(cnt + 1, &sub_polygon.into(), min_side_length));
    }

    polygons
}
pub fn generate_houses(district: &District, min_side_length: f64) -> Vec<Polygon<f64>> {
    assert!(!district.polygon().is_empty());

    let a = AnnotatedPolygon::new_adjacent_to_streets(district.polygon());
    foo(0, &a, min_side_length)
}
