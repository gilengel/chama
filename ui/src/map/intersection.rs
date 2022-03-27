use std::{cmp::Ordering, collections::HashMap, f64::consts::PI};

use geo::Coordinate;
use rust_editor::{
    gizmo::{GetPosition, Id, SetId, SetPosition},
    interactive_element::{InteractiveElement, InteractiveElementState},
    renderer::apply_style,
    style::{InteractiveElementStyle, Style},
};
use rust_macro::ElementId;
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use serde::{Deserialize, Serialize};

use super::street::Street;

#[derive(Clone, PartialEq, Copy, Debug, Serialize, Deserialize)]
pub enum Direction {
    In,
    Out,
}

#[derive(Clone, PartialEq, Copy, Debug)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Serialize, Deserialize, ElementId)]
pub struct Intersection {
    id: Uuid,
    position: Coordinate<f64>,

    connected_streets: Vec<(Direction, Uuid)>,

    style: InteractiveElementStyle,
    state: InteractiveElementState,
}

impl InteractiveElement for Intersection {
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

impl SetPosition for Intersection {
    fn set_position(&mut self, position: Coordinate<f64>) {
        self.position = position;
    }
}

impl GetPosition for Intersection {
    fn position(&self) -> Coordinate<f64> {
        self.position
    }
}

impl Intersection {
    pub fn new(position: Coordinate<f64>) -> Intersection {
        Intersection {
            position,
            ..Default::default()
        }
    }

    pub fn new_with_id(position: Coordinate<f64>, id: Uuid) -> Intersection {
        Intersection {
            position,
            id,
            ..Default::default()
        }
    }

    pub fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        context.save();
        context.begin_path();
        context.arc(self.position.x, self.position.y, 1.0, 0.0, 2.0 * PI)?;

        apply_style(self.style(), context);
        context.fill();
        context.restore();

        /*
        let num = self.connected_streets.len();
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
        */

        Ok(())
    }

    pub fn remove_connected_street(&mut self, id: &Uuid) -> Option<(Direction, Uuid)> {
        if let Some(index) = self
            .connected_streets
            .iter()
            .position(|x| x.1 == id.clone())
        {
            return Some(self.connected_streets.remove(index));
        }

        None
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
            (vec.y * 1.).atan2(vec.x) //+ (PI / 2.0)
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
            if let (Some(street_1), Some(street_2)) = (streets.get(&a.1), streets.get(&b.1)) {
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
                    if let Some(street_borrowed) = streets.get_mut(&id) {
                        street_borrowed.set_next(Side::Right, None);
                        street_borrowed.set_next(Side::Left, None);

                        if street_borrowed.id() != *previous_street {
                            street_borrowed.set_next(Side::Right, Some(*previous_street));
                        }

                        if street_borrowed.id() != *next_street {
                            street_borrowed.set_next(Side::Left, Some(*next_street));
                        }
                    }
                }

                Direction::Out => {
                    let street_borrowed = streets.get_mut(&id).unwrap();
                    street_borrowed.set_previous(Side::Right, None);
                    street_borrowed.set_previous(Side::Left, None);

                    if street_borrowed.id() != *next_street {
                        street_borrowed.set_previous(Side::Right, Some(*next_street));
                    }

                    if street_borrowed.id() != *previous_street {
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
            style: InteractiveElementStyle {
                normal: Style {
                    border_width: 1,
                    border_color: "#1e88e5".to_string(),
                    background_color: "#2A2A2B".to_string(),
                },
                hover: Style {
                    border_width: 0,
                    border_color: "".to_string(),
                    background_color: "#1e88e5".to_string(),
                },
                selected: Style {
                    border_width: 0,
                    border_color: "".to_string(),
                    background_color: "#00FFCC".to_string(),
                },
            },
            state: InteractiveElementState::Normal,
        }
    }
}
