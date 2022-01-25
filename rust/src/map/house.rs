use geo::{
    dimensions::HasDimensions,
    line_intersection::{line_intersection, LineIntersection},
    prelude::EuclideanDistance,
    winding_order::Winding,
    Coordinate, Line, LineString, Point, Polygon,
};

use super::district::District;


#[derive(Clone, Copy)]
pub struct LineSegment {
    pub start: Point<f64>,
    pub end: Point<f64>,
    pub norm: Point<f64>,
    pub perp: Point<f64>,
    pub length: f64,
}

impl LineSegment {
    pub fn new(start: Point<f64>, end: Point<f64>) -> LineSegment {
        let length = start.euclidean_distance(&end);
        let vec = end - start;
        let norm = Point::new(vec.x() / length, vec.y() / length);
        let perp = Point::new(-norm.y(),norm.x());

        LineSegment {
            start: start,
            end: end,
            norm,
            perp,
            length,
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

fn longest_segment(segments: &Vec<LineSegment>) -> &LineSegment {
    segments
        .into_iter()
        .max_by(|x, y| x.length.partial_cmp(&y.length).unwrap())
        .unwrap()
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
    let split_point = longest_segment.split_point() ;

    let mut intersections: Vec<PolygonLineIntersection> = Vec::new();

    let line = Line::new(split_point + longest_segment.perp * 600.0, split_point + longest_segment.perp * -600.0);

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
            None => {},
        };

        
    }

    pairs
}

struct Crossback {
    pub points: Vec<Point<f64>>,
    pub crossback: Option<usize>,
}

fn calc_split_polygons(
    polygon: &Polygon<f64>,
    intersections: &Vec<PolygonLineIntersection>,
) -> Vec<Polygon<f64>> {
    let points: Vec<Point<f64>> = polygon.exterior().points_cw().collect();

    let mut result: Vec<Crossback> = Vec::new();

    let intersection_pairs = calc_intersection_pairs(intersections);

    let mut current_index = 0;

    result.push(Crossback {
        points: Vec::new(),
        crossback: None,
    });
    for (i, point) in points.iter().enumerate() {
        result[current_index].points.push(*point);

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
                    .push(intersection.intersection.into());
                result[current_index].crossback = other_point_index;

                    match result
                    .iter()
                    .position(|x| x.crossback.is_some() && x.crossback.unwrap() == intersection.line_segment_index)
                {
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
                    .push(intersection.intersection.into());
            }
        }
    }

    result
        .iter()
        .map(|x| Polygon::new(LineString::from(x.points.clone()), vec![]))
        .collect()
}


fn foo(cnt: u32, polygon: &Polygon<f64>) -> Vec<Polygon<f64>> {
    let mut polygons : Vec<Polygon<f64>> = Vec::new();

    if cnt == 4 {
        polygons.push(polygon.clone());
        return polygons;
    }

    let mut segments: Vec<LineSegment> = vec![];
    let mut it = polygon.exterior().points_cw().peekable();
    let mut last_point = it.next().unwrap();

    for point in it {
        segments.push(LineSegment::new(last_point, point));

        last_point = point;
    }

    let longest_segment = longest_segment(&segments);
    let intersections = intersections(longest_segment, &segments);

    for sub_polygon in calc_split_polygons(polygon, &intersections) {
        polygons.append(&mut foo(cnt + 1, &sub_polygon));
    }

    polygons

}
pub fn generate_houses(district: &District) -> Vec<Polygon<f64>> {
    assert!(!district.polygon().is_empty());

    foo(0, district.polygon())    
}

