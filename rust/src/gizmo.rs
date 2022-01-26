use std::{collections::HashMap};

use geo::{Coordinate, Line, Triangle};
use uuid::Uuid;
use wasm_bindgen::{JsValue, UnwrapThrowExt};

pub enum Axis {
    X,
    Y,
    XY,
}

use crate::{renderer::PrimitiveRenderer, style::Style, Camera};

pub trait GetPosition {
    fn position(&self) -> Coordinate<f64>;
}

pub trait SetPosition {
    fn set_position(&mut self, position: Coordinate<f64>);
}

pub trait Id {
    fn id(&self) -> Uuid;
}

pub trait SetId {
    fn set_id(&mut self, id: Uuid);
}

pub trait Gizmo<'a, T: 'a>: GetPosition + SetPosition
where
    T: GetPosition + SetPosition,
{
    fn mouse_down(
        &mut self,
        mouse_pos: Coordinate<f64>,
        button: u32,
        elements: impl Iterator<Item = &'a mut T>,
    );
    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, elements: impl Iterator<Item = &'a mut T>);
    fn mouse_up(
        &mut self,
        mouse_pos: Coordinate<f64>,
        button: u32,
        elements: impl Iterator<Item = &'a mut T>,
    );
    fn render(
        &self,
        context: &web_sys::CanvasRenderingContext2d,
        camera: &Camera,
        elements: impl Iterator<Item = &'a T>,
    ) -> Result<(), JsValue>;
}

impl GetPosition for MoveGizmo {
    fn position(&self) -> Coordinate<f64> {
        self.position
    }
}

impl SetPosition for MoveGizmo {
    fn set_position(&mut self, position: Coordinate<f64>) {
        self.position = position;
    }
}

pub fn mouse_over(mouse_pos: Coordinate<f64>, origin: Coordinate<f64>) -> bool {
    let diff = mouse_pos - origin;

    if !(diff.x >= 0. && diff.x <= LINE_LENGTH && diff.y <= 0. && diff.y >= -LINE_LENGTH) {
        return false;
    }

    true
}

impl<'a, T: GetPosition + SetPosition + Id + 'a> Gizmo<'a, T> for MoveGizmo {
    fn mouse_down(
        &mut self,
        mouse_pos: Coordinate<f64>,
        button: u32,
        elements: impl Iterator<Item = &'a mut T>,
    ) {
        self.cursor_to_element_offset = mouse_pos - self.position();
        for x in elements {
            self.offsets.insert(x.id(), self.position() - x.position());
        }

        self.active = true;
        self.affected_axis(mouse_pos, self.position());
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        elements: impl Iterator<Item = &'a mut T>,
    ) {
        if !self.active {
            return;
        }

        for x in elements {
            let offset = self.offsets.get(&x.id()).unwrap_throw();

            let old_position = x.position();
            let new_position = match self.affected_axis.as_ref().unwrap() {
                crate::gizmo::Axis::X => Coordinate {
                    x: mouse_pos.x - offset.x - self.cursor_to_element_offset.x,
                    y: old_position.y,
                },
                crate::gizmo::Axis::Y => Coordinate {
                    x: old_position.x,
                    y: mouse_pos.y - offset.y - self.cursor_to_element_offset.y,
                },
                crate::gizmo::Axis::XY => mouse_pos - *offset - self.cursor_to_element_offset,
            };

            x.set_position(new_position);
        }

        self.set_position(Coordinate {
            x: mouse_pos.x - self.cursor_to_element_offset.x,
            y: mouse_pos.y - self.cursor_to_element_offset.y,
        });
    }

    fn mouse_up(
        &mut self,
        mouse_pos: Coordinate<f64>,
        button: u32,
        elements: impl Iterator<Item = &'a mut T>,
    ) {
        if self.active {
            self.active = false;
        }
    }

    fn render(
        &self,
        context: &web_sys::CanvasRenderingContext2d,
        camera: &Camera,
        elements: impl Iterator<Item = &'a T>,
    ) -> Result<(), JsValue> {
        if elements.peekable().peek().is_some() {
            let position = self.position();
            context.translate(position.x + camera.x as f64, position.y + camera.y as f64)?;

            self.x_axis.render(&self.x_style, &context)?;
            self.x_arrow.render(&self.x_style, &context)?;
            self.y_axis.render(&self.y_style, &context)?;
            self.y_arrow.render(&self.y_style, &context)?;

            context.set_transform(1., 0., 0., 1., 0., 0.)?;
        }

        Ok(())
    }
}

static ARROW_WIDTH: f64 = 6.0;
static ARROW_HEIGHT: f64 = 10.0;
static LINE_LENGTH: f64 = 100.0;

pub struct MoveGizmo {
    position: Coordinate<f64>,

    x_style: Style,
    x_axis: Line<f64>,
    x_arrow: Triangle<f64>,

    y_style: Style,
    y_axis: Line<f64>,
    y_arrow: Triangle<f64>,

    cursor_to_element_offset: Coordinate<f64>,

    offsets: HashMap<Uuid, Coordinate<f64>>,

    active: bool,

    pub affected_axis: Option<Axis>,
}

impl MoveGizmo {
    pub fn new() -> Self {
        MoveGizmo {
            position: Coordinate { x: 0., y: 0. },
            affected_axis: None,
            active: false,

            cursor_to_element_offset: Coordinate { x: 0., y: 0. },

            offsets: HashMap::new(),

            x_axis: Line::new(
                Coordinate { x: 0., y: 0. },
                Coordinate {
                    x: LINE_LENGTH,
                    y: 0.,
                },
            ),
            x_style: Style {
                border_width: 2,
                border_color: "#C45D53".to_string(),
                background_color: "#C45D53".to_string(),
            },
            x_arrow: Triangle(
                Coordinate {
                    x: LINE_LENGTH - ARROW_HEIGHT,
                    y: -ARROW_WIDTH / 2.,
                },
                Coordinate {
                    x: LINE_LENGTH,
                    y: 0.,
                },
                Coordinate {
                    x: LINE_LENGTH - ARROW_HEIGHT,
                    y: ARROW_WIDTH / 2.,
                },
            ),

            y_axis: Line::new(
                Coordinate { x: 0., y: 0. },
                Coordinate {
                    x: 0.,
                    y: -LINE_LENGTH,
                },
            ),
            y_style: Style {
                border_width: 2,
                border_color: "#11CC80".to_string(),
                background_color: "#11CC80".to_string(),
            },
            y_arrow: Triangle(
                Coordinate {
                    x: -ARROW_WIDTH / 2.,
                    y: -LINE_LENGTH + ARROW_HEIGHT,
                },
                Coordinate {
                    x: 0.,
                    y: -LINE_LENGTH,
                },
                Coordinate {
                    x: ARROW_WIDTH / 2.,
                    y: -LINE_LENGTH + ARROW_HEIGHT,
                },
            ),
        }
    }

    fn affected_axis(&mut self, mouse_pos: Coordinate<f64>, origin: Coordinate<f64>) {
        let diff = mouse_pos - origin;

        if diff.x.abs() < 30. && diff.y.abs() < 30. {
            self.affected_axis = Some(Axis::XY);
            return;
        }

        if diff.x.abs() < 30. && diff.y < 0. && diff.y >= -LINE_LENGTH {
            self.affected_axis = Some(Axis::Y);
            return;
        }

        if diff.y.abs() < 30. && diff.x > 0. && diff.x <= LINE_LENGTH {
            self.affected_axis = Some(Axis::X);
            return;
        }
    }
}
