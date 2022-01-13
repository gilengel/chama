use std::{cell::RefCell, rc::Rc};

use geo::{prelude::Contains, Coordinate, LineString, Polygon};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{
    interactive_element::{InteractiveElement, InteractiveElementState},
    intersection::Side,
    street::Street,
    style::{InteractiveElementStyle, Style}, map::Map,
};

pub struct District {
    pub id: Uuid,
    polygon: Polygon<f64>,
    style: InteractiveElementStyle,
    state: InteractiveElementState,
}
impl Default for District {
    fn default() -> Self {
        District {
            id: Uuid::new_v4(),
            polygon: Polygon::new(LineString::from(vec![Coordinate { x: 0., y: 0. }]), vec![]),
            style: InteractiveElementStyle {
                normal: Style {
                    border_width: 0,
                    border_color: "#FF0000".to_string(),
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
}

impl District {
    pub fn is_point_on_district(&self, point: &Coordinate<f64>) -> bool {
        self.polygon.contains(point)
    }

    pub fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        let mut it = self.polygon.exterior().points_iter();
        let start = it.next().unwrap();
        let style = self.style();

        context.save();

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

        context.restore();

        Ok(())
    }
}

struct Enclosed {
    enclosed: bool,
    _streets: Vec<(Side, Uuid)>,
    points: Vec<Coordinate<f64>>,
}

pub fn create_district_for_street(side: Side, street: Uuid, map: &mut Map) -> Option<District> {
    let district = enclosed(side, street, map);

    if !district.enclosed {
        return None;
    }

    return Some(District {
        polygon: Polygon::new(LineString::from(district.points), vec![]),
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
                next = match street.get_next(side) {
                    Some(id) => Some(id),
                    None => None,
                };
                points.push(street.start());
            } else {
                next = match street.get_previous(side) {
                    Some(id) => Some(id),
                    None => None,
                };
                points.push(street.end());
            }

            
            if next.is_some()
                && ((street.start == map.street(&next.unwrap()).unwrap().start) ||
                    (street.end == map.street(&next.unwrap()).unwrap().end))
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

    Enclosed {
        enclosed: next.is_some() && next.unwrap() == start,
        _streets: streets,
        points: points,
    }
}
