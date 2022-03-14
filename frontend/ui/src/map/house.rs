use geo::{euclidean_length::EuclideanLength, Line, Point, Polygon};
use rand::{thread_rng, Rng};
use rust_editor::style::Style;

use crate::algorithm::geo::{longest_line, split, AnnotatedPolygon};

use super::district::House;

/*
#[derive(Clone)]
struct AnnotatedPolygon(Polygon<f64>, Vec<bool>);

impl AnnotatedPolygon {
    pub fn new(polygon: Polygon<f64>, parent: &AnnotatedPolygon) -> Self {
        AnnotatedPolygon(
            polygon,
            polygon.exterior().lines().map(|line| inner_line(parent.0, line))
        )
    }


}

fn matching_parent_line(polygon: AnnotatedPolygon, line: (&Line<f64>, bool)) -> Option<(Line<f64>, bool)> {
    if let Some(x) = polygon.0.exterior().lines().zip(polygon.1.iter()).find(|(parent_line, street)| parent_line.contains(line.0)) {
        return Some((x.0, *x.1));
    }

    if let Some(x) = polygon.0.exterior().lines().zip(polygon.1.iter()).find(|(parent_line, street)| parent_line.contains(line.0)) {
        return Some((x.0, *x.1));
    }

    None
}
*/

/*
pub fn longest_muu(polygon: &Polygon<f64>) -> Line<f64> {
    let lines = polygon.exterior().lines();
    let streets: Vec<Line<f64>> = lines.filter(|line| inner_line(polygon, line)).collect();

    match streets.iter().max_by(|x, y| {
        x.euclidean_length()
            .partial_cmp(&y.euclidean_length())
            .unwrap()
    }) {
        Some(line) => line.clone(),
        None => longest_line(polygon),
    }
}
*/

fn muu(cnt: u32, polygon: &AnnotatedPolygon, min_side_length: f64) -> Vec<AnnotatedPolygon> {
    let mut polygons: Vec<AnnotatedPolygon> = Vec::new();

    if cnt == 6 {
        polygons.push(polygon.clone());
        return polygons;
    }

    let line = longest_line(&polygon, min_side_length).0; //longest_muu(polygon);
    let vec = line.end_point() - line.start_point();
    let length = line.euclidean_length();
    let norm = Point::new(vec.x() / length, vec.y() / length);
    let perp = Point::new(-norm.y(), norm.x());

    let split_pt = line.start_point() + norm * length * 0.5;
    let split_line = Line::new(split_pt - perp * 6000.0, split_pt + perp * 6000.0);

    for sub_polygon in split(&polygon, &split_line).iter_mut() {
        polygons.append(&mut muu(cnt + 1, &sub_polygon, min_side_length));
    }

    polygons
}

pub fn generate_houses_from_polygon(polygon: &Polygon<f64>, min_side_length: f64) -> Vec<House> {
    let houses = muu(
        0,
        &AnnotatedPolygon(
            polygon.clone(),
            polygon.exterior().lines().map(|_| true).collect(),
        ),
        min_side_length
    );
    let polygons = houses.iter();
    //.filter(|p| polygon.contains(*p));
    //.filter(|p| p.exterior().lines().all(|l| !inner_line(polygon, &l)));

    let mut rng = thread_rng();
    polygons
        //.iter()
        .map(|sub_polygon| {
            let r: u8 = rng.gen_range(0..255);
            let g: u8 = rng.gen_range(0..255);
            let b: u8 = rng.gen_range(0..255);

            let line_styles: Vec<Style> = sub_polygon
                .lines()
                .iter()
                .map(|(_, is_street)| {
                    if *is_street {
                        return Style {
                            border_width: 4,
                            border_color: "#FFFFFF".to_string(),
                            background_color: "".to_string(),
                        };
                    }

                    Style {
                        border_width: 4,
                        border_color: "#FF0000".to_string(),
                        background_color: "".to_string(),
                    }
                })
                .collect();

            House {
                polygon: sub_polygon.0.clone(),
                line_styles: line_styles.clone(),
                style: Style {
                    border_width: 2,
                    border_color: "#FFFFFF".to_string(),
                    background_color: format!("rgba({},{},{}, 0.2)", r, g, b).to_string(),
                },
            }
        })
        .collect()

    //split(polygon)

    /*
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
    */
    /*
    // skip inner polygons for now
    if enclosed_inner_polygon(polygon) {
        return polygons;
    }
    */
}
