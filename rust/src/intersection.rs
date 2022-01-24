use std::{cmp::Ordering, collections::HashMap, f64::consts::PI};

use geo::Coordinate;
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{log, map::InformationLayer, street::Street};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Copy, Debug, Serialize, Deserialize)]
pub enum Direction {
    In,
    Out,
}

#[derive(Clone, PartialEq, Copy)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Intersection {
    pub id: Uuid,
    position: Coordinate<f64>,

    connected_streets: Vec<(Direction, Uuid)>,
}

impl Intersection {
    pub fn new(position: Coordinate<f64>) -> Intersection {
        Intersection {
            position,
            ..Default::default()
        }
    }

    pub fn set_position(&mut self, position: Coordinate<f64>) {
        self.position = position;
    }

    pub fn get_position(&self) -> Coordinate<f64> {
        self.position
    }

    pub fn render(
        &self,
        context: &CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
    ) -> Result<(), JsValue> {
        context.begin_path();
        context.arc(self.position.x, self.position.y, 5.0, 0.0, 2.0 * PI)?;

        let num = self.connected_streets.len();
        match num {
            0 => context.set_fill_style(&"#FF0000".into()),
            2 => context.set_fill_style(&"#0000FF".into()),
            3 => context.set_fill_style(&"#FFFFFF".into()),
            _ => context.set_fill_style(&"#00FF00".into()),
        }
        context.fill();

        if additional_information_layer.contains(&InformationLayer::Debug) && num != 2 {
            context.set_fill_style(&"#FFFFFF".into());

            context.fill_text(
                &format!(
                    "c={}, {}",
                    self.connected_streets.len(),
                    &self.id.to_string()[..2]
                )
                .to_string(),
                self.position.x,
                self.position.y - 80.0,
            )?;

            let mut y = self.position.y - 60.0;
            for street in &self.connected_streets {
                context.fill_text(
                    &format!("{:?} {}", street.0, &street.1.to_string()[..2]).to_string(),
                    self.position.x,
                    y,
                )?;

                y += 16.0;
            }
        }

        Ok(())
    }

    pub fn remove_connected_street(&mut self, id: &Uuid) {
        if let Some(index) = self
            .connected_streets
            .iter()
            .position(|x| x.1 == id.clone())
        {
            self.connected_streets.remove(index);
        } else {
            log!(":(");
        }
    }

    pub fn add_incoming_street(&mut self, id: &Uuid) {
        self.connected_streets.push((Direction::In, *id));
    }

    pub fn add_outgoing_street(&mut self, id: &Uuid) {
        self.connected_streets.push((Direction::Out, *id));
    }

    pub fn get_connected_streets(&self) -> &Vec<(Direction, Uuid)> {
        &self.connected_streets
    }

    pub fn reorder(&mut self, streets: &mut HashMap<Uuid, Street>) {
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

        let sort_ascending_by_angle = |a: &(Direction, Uuid), b: &(Direction, Uuid)| -> Ordering {
            let street_1 = streets.get(&a.1).unwrap();
            let street_2 = streets.get(&b.1).unwrap();

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
        };

        self.connected_streets.sort_by(sort_ascending_by_angle);

        for i in 0..self.connected_streets.len() {
            let (direction, id) = self.connected_streets[i];

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
                    let street_borrowed = streets.get_mut(&id).unwrap();
                    street_borrowed.set_next(Side::Right, None);
                    street_borrowed.set_next(Side::Left, None);

                    if street_borrowed.id != *previous_street {
                        street_borrowed.set_next(Side::Right, Some(*previous_street));
                    }

                    if street_borrowed.id != *next_street {
                        street_borrowed.set_next(Side::Left, Some(*next_street));
                    }
                }

                Direction::Out => {
                    let street_borrowed = streets.get_mut(&id).unwrap();
                    street_borrowed.set_previous(Side::Right, None);
                    street_borrowed.set_previous(Side::Left, None);

                    if street_borrowed.id != *next_street {
                        street_borrowed.set_previous(Side::Right, Some(*next_street));
                    }

                    if street_borrowed.id != *previous_street {
                        street_borrowed.set_previous(Side::Left, Some(*previous_street));
                    }
                }
            }
        }
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
