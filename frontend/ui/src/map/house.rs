use std::iter::Peekable;

use geo::{
    dimensions::HasDimensions,
    line_intersection::{line_intersection, LineIntersection},
    prelude::{Area, Contains, EuclideanDistance},
    Coordinate, Line, LineString, Point, Polygon,
};
use rand::{thread_rng, Rng};
use rust_editor::{log, style::Style};
use serde::{Deserialize, Serialize};

use super::district::House;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum LineSegmentType {
    Street,
    Backside,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
            start,
            end,
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
        Line::new(self.start.0, self.end.0)
    }
}

impl PartialEq for LineSegment {
    fn eq(&self, other: &LineSegment) -> bool {
        self.start == other.start && self.end == other.end
    }
}

fn longest_segment(segments: &Vec<LineSegment>, min_side_length: f64) -> &LineSegment {
    let adjacent_street_segments = segments.iter().filter(|segment| {
        segment.ty == LineSegmentType::Street && segment.length > min_side_length
    });

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

#[derive(PartialEq, Debug)]
enum PointType {
    Intersection,
    Street,
}

struct Crossback {
    pub points: Vec<(Point<f64>, PointType)>,
    pub crossback: Option<usize>,
}

impl Crossback {
    fn new() -> Crossback {
        Crossback {
            points: Vec::new(),
            crossback: None,
        }
    }
}

fn find_corresponding_other_segment_index(
    intersection: &PolygonLineIntersection,
    intersection_pair: &(PolygonLineIntersection, PolygonLineIntersection),
) -> usize {
    if intersection.line_segment_index == intersection_pair.0.line_segment_index {
        return intersection_pair.1.line_segment_index;
    }

    intersection_pair.0.line_segment_index
}

fn calc_split_polygons(
    segments: &Vec<LineSegment>,
    intersections: &Vec<PolygonLineIntersection>,
) -> Vec<AnnotatedPolygon> {
    let mut result: Vec<Crossback> = Vec::new();

    let intersection_pairs = calc_intersection_pairs(intersections);

    result.push(Crossback::new());
    let mut current_crossback: &mut Crossback = result.last_mut().unwrap();

    // Iterate over the polygon in clock wise order
    for (i, segment) in segments.iter().enumerate() {
        // Normal case, current segment was not intersected and we only need to add the next point to the current polygon
        current_crossback
            .points
            .push((segment.start, PointType::Street));

        // Special case, the current segment index matches an intersection therefore we need to jump to the crossback of it
        if let Some(intersection) = intersections.iter().find(|x| x.line_segment_index == i) {
            // We add intersection to the resulting polygon
            current_crossback
                .points
                .push((intersection.intersection.into(), PointType::Intersection));

            for k in intersection_pairs.iter() {
                current_crossback.crossback =
                    Some(find_corresponding_other_segment_index(intersection, k));

                // Find in all the crossbacks the one corresponding one if existing, otherwise create a new one
                match result.iter_mut().find(|other_crossback| {
                    other_crossback.crossback.is_some()
                        && other_crossback.crossback.unwrap() == intersection.line_segment_index
                }) {
                    Some(k) => current_crossback = k,
                    None => {
                        // Found a new subpolygon, create a new crossback and use it to generate it
                        result.push(Crossback::new());
                        current_crossback = result.last_mut().unwrap();
                    }
                }

                // And now we add the other corresponding intersection to the resulting polygon
                current_crossback
                    .points
                    .push((intersection.intersection.into(), PointType::Intersection));
            }
        }
    }

    result
        .iter()
        .map(|x| {
            fn bla(
                current_pt: &Point<f64>,
                current_type: &PointType,
                next_pt: &Point<f64>,
                next_type: &PointType,
            ) -> LineSegment {
                let ty = if *current_type == PointType::Intersection
                    && *next_type == PointType::Intersection
                {
                    LineSegmentType::Backside
                } else {
                    LineSegmentType::Street
                };

                LineSegment::new(*current_pt, *next_pt, ty)
            }

            let mut it = x.points.iter().peekable();

            let mut lines: Vec<LineSegment> = Vec::new();
            while let Some((current_pt, current_type)) = it.next() {
                match it.peek() {
                    Some((next_pt, next_type)) => {
                        lines.push(bla(current_pt, current_type, next_pt, next_type))
                    }

                    None => {
                        let (next_pt, next_type) = x.points.first().unwrap();
                        lines.push(bla(current_pt, current_type, next_pt, next_type));
                    }
                }
            }

            AnnotatedPolygon(lines)
        }) //Polygon::new(LineString::from(x.points.clone()), vec![]))
        .collect()
}

/*/
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
                    .push((intersection.intersection.into(), if segments[intersection.line_segment_index].street { PointType::StreetIntersection } else { PointType::Intersection }));
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

                let point_type = match intersection.ty {
                    LineSegmentType::Street => PointType::StreetIntersection,
                    LineSegmentType::Backside => PointType::Intersection,
                };
                result[current_index]
                    .points
                    .push((intersection.intersection.into(), point_type));
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
*/

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AnnotatedPolygon(Vec<LineSegment>);

impl AnnotatedPolygon {
    pub fn new_adjacent_to_streets(polygon: &Polygon<f64>) -> Self {
        AnnotatedPolygon(
            polygon
                .exterior()
                .lines()
                .map(|line| {
                    LineSegment::new(
                        line.start_point(),
                        line.end_point(),
                        LineSegmentType::Street,
                    )
                })
                .collect(),
        )
    }

    pub fn iter(&self) -> Peekable<std::slice::Iter<'_, LineSegment>> {
        self.0.iter().peekable()
    }

    pub fn enclosed_inner_polygon(&self) -> bool {
        log!(
            "{} / {}",
            self.0
                .iter()
                .filter(|segment| segment.ty == LineSegmentType::Backside)
                .count(),
            self.0.len()
        );
        
        self.0
                .iter()
                .all(|segment| segment.ty == LineSegmentType::Backside)
    }

    pub fn size(&self) -> f64 {
        let polygon: Polygon<f64> = self.into();
        polygon.unsigned_area()
    }

    pub fn match_segment_types_with_parent_polygon(&mut self, polygon: &AnnotatedPolygon) {
        self.0.iter_mut().for_each(|x| {
            x.ty = match polygon.0.iter().find(|parent_segment| {
                parent_segment.start == x.start || parent_segment.start == x.end || 
                parent_segment.end == x.start || parent_segment.end == x.end
            }) {
                Some(parent_segment) => parent_segment.ty,
                None => LineSegmentType::Backside,
            }
        });
    }
}

impl Into<Line<f64>> for LineSegment {
    fn into(self) -> Line<f64> {
        Line {
            start: self.start.into(),
            end: self.end.into(),
        }
    }
}

impl Into<Polygon<f64>> for &AnnotatedPolygon {
    fn into(self) -> Polygon<f64> {
        assert!(self.0.first().unwrap().start == self.0.last().unwrap().end);

        let pts: Vec<Point<f64>> = self
            .0
            .iter()
            .map(|line_segment| line_segment.start)
            .collect();

        Polygon::new(LineString::from(pts), vec![])
    }
}

fn foo(cnt: u32, polygon: &AnnotatedPolygon, min_side_length: f64) -> Vec<AnnotatedPolygon> {
    let line: Line<f64> = Line::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 100., y: 0. });

    let line2: Line<f64> = Line::new(Coordinate { x: 25., y: 0. }, Coordinate { x: 75., y: 0. });

    assert!(line.contains(&line2));

    let mut polygons: Vec<AnnotatedPolygon> = Vec::new();

    if polygon.size() < min_side_length * min_side_length {}
    if cnt == 3 {
        polygons.push(polygon.clone());
        return polygons;
    }

    let segments = &polygon.0;
    let longest_segment = longest_segment(&segments, min_side_length);
    let intersections = intersections(longest_segment, &segments);

    for sub_polygon in calc_split_polygons(&segments, &intersections).iter_mut() {
        sub_polygon.match_segment_types_with_parent_polygon(polygon);
        polygons.append(&mut foo(cnt + 1, &sub_polygon, min_side_length));
    }

    polygons
}

pub fn generate_houses_from_polygon(polygon: &Polygon<f64>, min_side_length: f64) -> Vec<House> {
    assert!(!polygon.is_empty());

    let mut rng = thread_rng();

    let a = AnnotatedPolygon::new_adjacent_to_streets(polygon);
    foo(0, &a, min_side_length)
        .iter()
        .filter(|sub_polygon| !sub_polygon.enclosed_inner_polygon())
        .map(|polygon| {
            let pts: Vec<Point<f64>> = polygon.0.iter().map(|pt| pt.start.clone()).collect();

            let styles: Vec<Style> = polygon
                .0
                .iter()
                .map(|_| Style {
                    border_width: 0,
                    border_color: "#FFFFFF".to_string(),
                    background_color: "#FF0000".to_string(),
                })
                .collect();

            let r = rng.gen_range(0..255);
            let g = rng.gen_range(0..255);
            let b = rng.gen_range(0..255);
            House {
                polygon: Polygon::new(LineString::from(pts), vec![]),
                point_style: styles,
                style: Style {
                    border_width: 1,
                    border_color: "#FFFFFF".to_string(),
                    background_color: format!("rgb({},{},{})", r, g, b).to_string(),
                },
            }
        })
        .collect()

    /*
    // skip inner polygons for now
    if enclosed_inner_polygon(polygon) {
        return polygons;
    }
    */
}
