use std::{cell::RefCell, rc::Rc};

use geo::{Coordinate, Polygon, LineString};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{intersection::Side, street::Street};

pub struct District {
    polygon: Polygon<f64>
}

impl District {
    pub fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        let mut it = self.polygon.exterior().points_iter();
        let start = it.next().unwrap();

        context.begin_path();
        context.move_to(start.x(), start.y());
        for point in it {
            context.line_to(point.x(), point.y());
        }

        context.close_path();
        context.set_fill_style(&"#FF2A2B".into());
        context.fill();

        Ok(())
    }    
}

struct Enclosed {
    enclosed: bool,
    _streets: Vec<(Side, Rc<RefCell<Street>>)>,
    points: Vec<Coordinate<f64>>,
}

pub fn create_district_for_street(street: Rc<RefCell<Street>>) -> (Option<District>, Option<District>) {
    let left = enclosed(Side::Left, street);

    let mut result: (Option<District>, Option<District>) = (None, None);
    if left.enclosed {
        result.0 = Some(District { polygon:  Polygon::new(LineString::from(left.points), vec![]) });
    }


    result
}

fn enclosed(side: Side, starting_street: Rc<RefCell<Street>>) -> Enclosed {
    let mut side = side;
    let start = starting_street.as_ref().borrow();

    let mut next: Option<Rc<RefCell<Street>>> = match start.get_next(side) {
        Some(n) => Some(Rc::clone(n)),
        None => None,
    };

    let mut street = Rc::clone(&starting_street);
    let mut forward = true;

    let mut streets: Vec<(Side, Rc<RefCell<Street>>)> = vec![];
    let mut points: Vec<Coordinate<f64>> = vec![];

    
    while next.is_some() && !Rc::ptr_eq(&next.as_ref().unwrap(), &starting_street) {
        streets.push((side, Rc::clone(&street)));

        {
            let street = street.as_ref().borrow();
            
            if forward {
                next = match street.get_next(side) {
                    Some(n) => Some(Rc::clone(n)),
                    None => None,
                };
                points.push(street.start());
            } else {
                next = match street.get_previous(side) {
                    Some(n) => Some(Rc::clone(n)),
                    None => None,
                };
                points.push(street.end());
            }

            if next.is_some()
                && (Rc::ptr_eq(
                    street.start.as_ref().unwrap(),
                    &next.as_ref().unwrap().borrow().start.as_ref().unwrap(),
                ) || Rc::ptr_eq(
                    street.end.as_ref().unwrap(),
                    &next.as_ref().unwrap().borrow().end.as_ref().unwrap(),
                ))
            {
                forward = !forward;

                side = match side {
                    Side::Left => Side::Right,
                    Side::Right => Side::Left,
                }
            }
        }

        if next.is_some() {
            street = Rc::clone(&next.as_ref().unwrap());
        }
    }

    Enclosed {
        enclosed: next.is_some() && Rc::ptr_eq(&next.as_ref().unwrap(), &starting_street),
        _streets: streets,
        points: points,
    }
}
