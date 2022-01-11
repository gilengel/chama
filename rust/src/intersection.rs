

use geo::{Coordinate};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{street::Street};

#[derive(Clone, PartialEq, Copy)]
pub enum Direction {
    In,
    Out,
}

#[derive(Clone, PartialEq, Copy)]
pub enum Side {
    Left,
    Right,
}

/*
#[derive(PartialEq)]
enum Adjacency {
    Previous,
    Next,
}
*/

#[derive(Clone)]
pub struct Intersection {
    pub id: Uuid,
    position: Coordinate<f64>,

    connected_streets: Vec<(Direction, Uuid)>,
}


impl Intersection {
    pub fn new(
        position: Coordinate<f64>,
        connected_streets: Vec<(Direction, Uuid)>,
    ) -> Intersection {
        Intersection {
            id: Uuid::new_v4(),
            position,
            connected_streets,
        }
    }

    pub fn set_position(&mut self, position: Coordinate<f64>) {
        self.position = position;
    }

    pub fn get_position(&self) -> Coordinate<f64> {
        self.position
    }

    pub fn render(&self, _context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        /*
        context.begin_path();
        context.arc(self.position.x, self.position.y, 15.0, 0.0, 2.0 * PI)?;
        context.set_fill_style(&"#FF8C00".into());
        context.fill();

        
        context.set_fill_style(&"#FFFFFF".into());

        
        context.fill_text(
            &format!("c={}", self.connected_streets.len()).to_string(),
            self.position.x,
            self.position.y - 20.0,
        )?;
        

        let mut y = self.position.y;
        for street in &self.connected_streets {
            let street = street.1.as_ref().borrow();
            context.fill_text(&format!("{}", street.id).to_string(), self.position.x, y)?;

            y += 16.0;
        }
        */

        Ok(())
    }

    pub fn remove_connected_street(&mut self, street: &Street) {
        self.connected_streets.retain(|&x| x.1 == street.id);
    }

    pub fn is_connected_to_street(&self, street: &Street) -> bool {
        self.connected_streets
            .iter()
            .any(|x| x.1 == street.id)
    }

    pub fn add_incoming_street(&mut self, street: &Street) {
        self.connected_streets.push((Direction::In, street.id));
    }

    pub fn add_outgoing_street(&mut self, street: &Street) {
        self.connected_streets.push((Direction::Out, street.id));
    }

    pub fn get_connected_streets(&self) -> &Vec<(Direction, Uuid)> {
        &self.connected_streets
    }

    pub fn reorder(&mut self) {
        // TODO
        /*
        fn angle(vec: &Coordinate<f64>) -> f64 {
            vec.y.atan2(vec.x) + (PI / 2.0)
        }

        fn norm_based_on_direction(direction: Direction, street: &Street) -> Coordinate<f64> {
            let norm = if direction == Direction::Out {
                street.norm()
            } else {
                street.inverse_norm()
            };

            norm
        }

        fn sort_ascending_by_angle(
            a: &(Direction, Rc<RefCell<Street>>),
            b: &(Direction, Rc<RefCell<Street>>),
        ) -> Ordering {
            let street_1 = (*a.1).borrow();
            let street_2 = (*b.1).borrow();

            let norm_1 = norm_based_on_direction(a.0, &street_1);
            let norm_2 = norm_based_on_direction(b.0, &street_2);

            let angle_1 = angle(&norm_1);
            let angle_2 = angle(&norm_2);
            if angle_1 < angle_2 {
                return Ordering::Less;
            }

            if angle_1 > angle_2 {
                return Ordering::Greater;
            }

            Ordering::Equal
        }

        self.connected_streets.sort_by(sort_ascending_by_angle);
        
        for i in 0..self.connected_streets.len() {
            let (direction, street) = self.connected_streets[i].borrow();
      
            let (_, previous_street) = if i > 0 {
                &self.connected_streets[i - 1]
            } else {
                self.connected_streets.last().unwrap()
            };

            let (_, next_street) = if i < self.connected_streets.len() - 1 {
                &self.connected_streets[i + 1]
            } else {
                self.connected_streets.first().unwrap()
            };

            match direction {
                Direction::In => {
                    let mut street_borrowed= street.borrow_mut();
                    street_borrowed.set_next(Side::Right, None);
                    street_borrowed.set_next(Side::Left, None);

                    
                    if !Rc::ptr_eq(street, previous_street) {
                        street_borrowed.set_next(Side::Right, Some(Rc::clone(previous_street)));
                    }                    

                    if !Rc::ptr_eq(street, next_street) {
                        street_borrowed.set_next(Side::Left, Some(Rc::clone(next_street)));
                    }
                }

                Direction::Out => {
                    let mut street_borrowed= street.borrow_mut();
                    street_borrowed.set_previous(Side::Right, None);
                    street_borrowed.set_previous(Side::Left, None);

                    if !Rc::ptr_eq(street, next_street) {
                        street_borrowed.set_previous(Side::Right, Some(Rc::clone(next_street)));
                    }

                    if !Rc::ptr_eq(street, previous_street) {
                        street_borrowed.set_previous(Side::Left, Some(Rc::clone(previous_street)));
                    }
                }
            }            
        }
        */
    }
}

impl Default for Intersection {
    fn default() -> Self {
        Intersection {
            id: Uuid::new_v4(),
            position: Coordinate { x: 0., y: 0. },
            connected_streets: vec![],
        }
    }
}
