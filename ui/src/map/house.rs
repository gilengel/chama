use geo::{euclidean_length::EuclideanLength, prelude::Area, Line, Point, Polygon};
use rand::{thread_rng, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rust_editor::{style::Style, log};

use crate::algorithm::geo::{longest_line, split, AnnotatedPolygon, longest_and_shortest_diameter};

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

fn calculate_split_line(rng: &mut ChaCha8Rng, polygon: &AnnotatedPolygon, min_side_length: f64) -> Line<f64> {
    let line = longest_line(&polygon, min_side_length).0; //longest_muu(polygon);
    let vec = line.end_point() - line.start_point();
    let length = line.euclidean_length();
    let norm = Point::new(vec.x() / length, vec.y() / length);
    let perp = Point::new(-norm.y(), norm.x());

    let split_pt = line.start_point() + norm * length * 0.70; //* rng.gen_range(0.3..0.7);

    
    
    Line::new(split_pt - perp * 6000.0, split_pt + perp * 6000.0)
}

fn split_polygons_into_chunks(rng: &mut ChaCha8Rng, polygon: &AnnotatedPolygon, min_side_length: f64, cnt: u8, max_cnt: u8) -> Vec<AnnotatedPolygon> {
    let mut polygons: Vec<AnnotatedPolygon> = Vec::new();

    let p = longest_and_shortest_diameter(&polygon.0);
  
    //if polygon.0.unsigned_area() < min_side_length * min_side_length {
    if p.0 < min_side_length && p.1 < min_side_length {
        polygons.push(polygon.clone());
        return polygons;
    }

    let split_line = calculate_split_line(rng, polygon, min_side_length);
    let mut splits = split(&polygon, &split_line);
    for sub_polygon in splits.iter_mut() {
        polygons.append(&mut split_polygons_into_chunks(rng, &sub_polygon, min_side_length, cnt + 1, max_cnt));
    }

    polygons
}

pub fn generate_houses_from_polygon(polygon: &Polygon<f64>, min_side_length: f64, seed: <ChaCha8Rng as SeedableRng>::Seed) -> Vec<House> {
    thread_rng().fill(&mut seed.clone());
    let mut rng = ChaCha8Rng::from_seed(seed);

    let houses = split_polygons_into_chunks(
        &mut rng,
        &AnnotatedPolygon(
            polygon.clone(),
            polygon.exterior().lines().map(|_| true).collect(),
        ),
        min_side_length,
        0,
        4
    );

    let polygons = houses.iter().filter(|polygon| !polygon.enclosed());

    
    polygons
        .map(|sub_polygon| {
            /*
            let r: u8 = rng.gen_range(0..255);
            let g: u8 = rng.gen_range(0..255);
            let b: u8 = rng.gen_range(0..255);

            let rng_color = format!("rgba({},{},{}, 0.3)", r, g, b).to_string();
            */

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
                        border_color: "#FFFFFF".to_string(),
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
                    background_color: "rgba(255, 255, 255, 0.3)".to_string(),
                },
            }
        })
        .collect()
}
