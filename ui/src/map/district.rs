use geo::{
    prelude::{Centroid, Contains},
    Coordinate, LineString, Polygon,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_editor::{gizmo::SetId, renderer::PrimitiveRenderer, log};
use rust_editor::{
    gizmo::Id,
    interactive_element::{InteractiveElement, InteractiveElementState},
    style::{InteractiveElementStyle, Style},
};
use rust_macro::ElementId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d; // TODO

use super::{house::generate_houses_from_polygon, map::Map};
use crate::map::intersection::Side;

#[derive(Serialize, Deserialize)]
pub struct House {
    pub polygon: Polygon<f64>,
    pub line_styles: Vec<Style>,
    pub style: Style,
}

#[derive(Serialize, Deserialize, ElementId)]
pub struct District {
    id: Uuid,
    polygon: Polygon<f64>,
    style: InteractiveElementStyle,
    state: InteractiveElementState,
    pub minimum_house_side: f64,
    pub seed: <ChaCha8Rng as SeedableRng>::Seed,

    houses: Vec<House>,
}

impl Default for District {
    fn default() -> Self {
        District {
            id: Uuid::new_v4(),
            polygon: Polygon::new(LineString::from(vec![Coordinate { x: 0., y: 0. }]), vec![]),
            style: InteractiveElementStyle {
                normal: Style {
                    border_width: 0,
                    border_color: "#000000".to_string(),
                    background_color: "rgba(30, 136, 229, 0.4)".to_string(),
                },
                hover: Style {
                    border_width: 0,
                    border_color: "".to_string(),
                    background_color: "#1e88e5".to_string(),
                },
                selected: Style {
                    border_width: 0,
                    border_color: "".to_string(),
                    background_color: "hsl(0, 100%, 50%)".to_string(),
                },
            },
            state: InteractiveElementState::Normal,
            minimum_house_side: 500.0,
            houses: Vec::new(),
            seed: Default::default()
        }
    }
}

impl InteractiveElement for District {
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
        self.state.clone()
    }
}

impl District {
    pub fn is_point_on_district(&self, point: &Coordinate<f64>) -> bool {
        self.polygon.contains(point)
    }

    pub fn polygon(&self) -> &Polygon<f64> {
        &self.polygon
    }

    pub fn update_houses(&mut self) {
        self.houses = generate_houses_from_polygon(&self.polygon, self.minimum_house_side, self.seed);
    }

    pub fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        //let mut it = self.polygon.exterior().points();
        //let start = it.next().unwrap();
        //let style = self.style();

        context.save();

        /*
        context.begin_path();
        context.move_to(start.x(), start.y());
        for point in it {
            context.line_to(point.x(), point.y());
        }

        context.close_path();
        context.set_fill_style(&style.background_color.clone().into());
        context.fill();


        if style.border_width > 0 {
            context.set_line_width(style.border_width.into());
            context.set_stroke_style(&style.border_color.clone().into());
            context.stroke();
        }
        */

        /*
        fn muu(ty: LineSegmentType) -> Style {
            if ty == LineSegmentType::Street { Style {
                border_width: 2,
                border_color: "#C45D53".to_string(),
                background_color: "#C45D53".to_string(),
            }
            } else {
                Style {
                    border_width: 2,
                    border_color: "#C45D53".to_string(),
                    background_color: "#C45D53".to_string(),
                }
            }
        }
*/
        for p in &self.houses {
            p.polygon.render(&p.style, context)?;

            for (line, style) in p.polygon.exterior().lines().zip(p.line_styles.iter()) {
                line.render(style, context)?;
            }
        }
        

        context.restore();

        Ok(())
    }
}

struct Enclosed {
    enclosed: bool,
    _streets: Vec<(Side, Uuid)>,
    points: Vec<Coordinate<f64>>,
}

pub fn create_district_for_street(side: Side, street: Uuid, map: &mut Map, minimum_house_side: f64, seed: <ChaCha8Rng as SeedableRng>::Seed) -> Option<District> {
    let district = enclosed(side, street, map);

    let factor = match side {
        Side::Left => -1.0,
        Side::Right => 1.0,
    };
    let street_perp = map.street(&street).unwrap().perp();
    let street_start: Coordinate<f64> = map.street(&street).unwrap().line.centroid().into();
    let intersected_streets = map.streets_intersecting_ray(
        &(street_start + street_perp * 10.0 * factor),
        &street_perp,
        10000.0 * factor,
    );


    if !district.enclosed || intersected_streets.is_empty() {
        return None;
    }

    // Generate the houses
    let polygon = Polygon::new(LineString::from(district.points), vec![]);
    let houses: Vec<House> = generate_houses_from_polygon(&polygon, minimum_house_side, seed);

    return Some(District {
        polygon,
        houses,
        minimum_house_side,
        ..District::default()
    });
}

fn enclosed(side: Side, street: Uuid, map: &mut Map) -> Enclosed {
    let mut side = side;
    let start = street;

    let mut next = match map.street(&street).unwrap().get_next(side) {
        Some(id) => Some(id),
        None => None,
    };

    let mut street = start;
    let mut forward = true;

    let mut streets: Vec<(Side, Uuid)> = vec![];
    let mut points: Vec<Coordinate<f64>> = vec![];

    while next.is_some() && next.unwrap() != start {
        streets.push((side, street));

        {
            let street = map.street(&street).unwrap();

            if forward {
                next = street.get_next(side);
                points.push(street.start());
            } else {
                next = street.get_previous(side);
                points.push(street.end());
            }

            if next.is_some()
                && ((street.start == map.street(&next.unwrap()).unwrap().start)
                    || (street.end == map.street(&next.unwrap()).unwrap().end))
            {
                forward = !forward;

                side = match side {
                    Side::Left => Side::Right,
                    Side::Right => Side::Left,
                }
            }
        }

        if let Some(next) = next {
            street = next;
        }
    }

    log!("{:?}", streets);
    Enclosed {
        enclosed: next.is_some() && next.unwrap() == start,
        _streets: streets,
        points: points,
    }
}
