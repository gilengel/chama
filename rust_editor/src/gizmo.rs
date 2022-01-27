use std::{collections::HashMap};

use geo::{Coordinate, Line, Triangle, prelude::EuclideanDistance};
use uuid::Uuid;
use wasm_bindgen::{JsValue, UnwrapThrowExt};

use crate::{camera::Camera, style::Style, renderer::PrimitiveRenderer};

pub enum Axis {
    X,
    Y,
    XY,
}

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
        _button: u32,
        elements: impl Iterator<Item = &'a mut T>,
    ) {
        self.cursor_to_element_offset = mouse_pos - self.position();
        for x in elements {
            self.offsets.insert(x.id(), self.position() - x.position());
        }

        self.active = mouse_over(mouse_pos, self.position());

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
        _mouse_pos: Coordinate<f64>,
        _button: u32,
        _elements: impl Iterator<Item = &'a mut T>,
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

            self.x_handle.render(&context)?;
            self.y_handle.render(&context)?;

            context.set_transform(1., 0., 0., 1., 0., 0.)?;
        }

        Ok(())
    }
}

static ARROW_WIDTH: f64 = 6.0;
static ARROW_HEIGHT: f64 = 10.0;
static LINE_LENGTH: f64 = 100.0;

struct GizmoArrow {
    style: Style,
    line: Line<f64>,
    arrow: Triangle<f64>
}

impl GizmoArrow {
    pub fn new(line_end: Coordinate<f64>, style: Style) -> Self {
        let len = line_end.euclidean_distance(&Coordinate { x: 0., y: 0.});
        let norm = Coordinate { x: line_end.x / len, y: line_end.y / len };
        let perp = Coordinate { x: -norm.y, y: norm.x };

        GizmoArrow {
            line: Line::new(
                Coordinate { x: 0., y: 0. },
                line_end
            ),
            style,
            arrow: Triangle(
                norm * (len - ARROW_HEIGHT) - perp * ARROW_WIDTH / 2.,
                line_end,
                norm * (len - ARROW_HEIGHT) + perp * ARROW_WIDTH / 2.
            ),            
        }
    }

    pub fn render(&self, context: &web_sys::CanvasRenderingContext2d) -> Result<(), JsValue> {
        self.line.render(&self.style, context)?;
        self.arrow.render(&self.style, context)?;

        Ok(())
    }
}


pub struct MoveGizmo {
    position: Coordinate<f64>,

    x_handle: GizmoArrow,
    y_handle: GizmoArrow,

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

            x_handle: GizmoArrow::new(Coordinate {
                x: 0.,
                y: -LINE_LENGTH,
            },Style {
                border_width: 2,
                border_color: "#C45D53".to_string(),
                background_color: "#C45D53".to_string(),
            }),

            y_handle: GizmoArrow::new(Coordinate {
                x: LINE_LENGTH,
                y: 0.,
            },Style {
                border_width: 2,
                border_color: "#C45D53".to_string(),
                background_color: "#C45D53".to_string(),
            }),
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

    pub fn is_active(&self) -> bool {
        self.active
    }
}
