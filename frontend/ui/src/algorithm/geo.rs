use geo::{
    coords_iter::CoordsIter,
    euclidean_length::EuclideanLength,
    line_intersection::{line_intersection, LineIntersection},
    prelude::Contains,
    Coordinate, Line, LineString, Point, Polygon,
};

type AnnotatedLine<'a> = (Line<f64>, &'a bool);

impl AnnotatedPolygon {
    pub(crate) fn lines(
        &self,
    ) -> Vec<AnnotatedLine> {
        self.0.exterior().lines().zip(self.1.iter()).collect()
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
    lines: &Vec<AnnotatedLine>
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

struct Crossback<'a> {
    pub lines: Vec<(Line<f64>, &'a bool)>,
    pub crossback: Option<usize>,
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

                    let start = if i == 0 {
                        line.start
                    } else {
                        result[current_index].lines.last().unwrap().0.end
                    };

                    result[current_index]
                        .lines
                        .push((Line::new(start, intersection.intersection), lines[intersection.line_segment_index].1));
                    result[current_index].crossback = other_point_index;

                    match result.iter().position(|x| {
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

                    result[current_index]
                        .lines
                        .push((Line::new(intersection.intersection, line.end), lines[intersection.line_segment_index].1));
                }
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

/*
fn calc_split_polygons(
    polygon: &Polygon<f64>,
    intersections: &Vec<PolygonLineIntersection>,
) -> Vec<Polygon<f64>> {
    let points: Vec<Point<f64>> = polygon.exterior().points().collect();

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

                match result.iter().position(|x| {
                    x.crossback.is_some() && x.crossback.unwrap() == intersection.line_segment_index
                }) {
                    Some(k) => {
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
*/
pub fn longest_line(polygon: &AnnotatedPolygon) -> AnnotatedLine {
    *polygon
        .lines()
        .iter()
        .max_by(|x, y| {
            x.0.euclidean_length()
                .partial_cmp(&y.0.euclidean_length())
                .unwrap()
        })
        .unwrap()
}

pub fn inner_line(polygon: &Polygon<f64>, line: &Line<f64>) -> bool {
    polygon.exterior().lines().any(|l| l.contains(line))
        || polygon.exterior_coords_iter().any(|pt| pt == line.start)
        || polygon.exterior_coords_iter().any(|pt| pt == line.end)
}