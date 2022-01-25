use std::fmt::Debug;

use geo::{Coordinate, Line, Triangle};
use uuid::Uuid;
use wasm_bindgen::JsValue;

pub enum Axis {
    X,
    Y,
    XY,
}

use crate::{
    intersection::Intersection, log, map::Map, renderer::PrimitiveRenderer, style::Style, Camera,
};

pub trait GetPosition {
    fn position(&self) -> Coordinate<f64>;
}

pub trait SetPosition {
    fn set_position(&mut self, position: Coordinate<f64>);
}

pub trait Gizmo<'a, T>
where
    T: GetPosition + SetPosition,
{
    fn mouse_over(&mut self, mouse_pos: Coordinate<f64>, origin: Coordinate<f64>) -> bool;
    fn mouse_down(&mut self, mouse_pos: Coordinate<f64>, button: u32, map: &mut Map);
    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map);
    fn mouse_up(&mut self, mouse_pos: Coordinate<f64>, button: u32, map: &mut Map);
    fn render(
        &self,
        context: &web_sys::CanvasRenderingContext2d,
        camera: &Camera,
        map: &Map,
    ) -> Result<(), JsValue>;

    fn element<F>(&'a self, id: Uuid, map: &'a Map, callback: F) -> &T
    where
        F: Fn(Uuid, &'a Map) -> &'a T;

    fn element_mut<F>(&'a self, id: Uuid, map: &'a mut Map, callback: F) -> &mut T
    where
        F: Fn(Uuid, &'a mut Map) -> &'a mut T;
}

impl<'a, T: GetPosition + SetPosition> Gizmo<'a, T> for MoveGizmo<T> {
    fn mouse_over(&mut self, mouse_pos: Coordinate<f64>, origin: Coordinate<f64>) -> bool {
        let diff = mouse_pos - origin;

        if !(diff.x >= 0. && diff.x <= LINE_LENGTH && diff.y <= 0. && diff.y >= -LINE_LENGTH) {
            return false;
        }

        true
    }

    fn mouse_down(&mut self, mouse_pos: Coordinate<f64>, button: u32, map: &mut Map) {
        if self.element_id.is_none() {
            return;
        }

        let element_id = self.element_id.unwrap();
        let origin = self.element(element_id, map, self.callback).position();

        if !self.mouse_over(mouse_pos, origin) {
            return;
        }

        self.cursor_to_element_offset = mouse_pos - origin;

        self.active = true;
        self.affected_axis(mouse_pos, origin);
    }

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map) {
        if !self.active {
            return;
        }

        let element_id = self.element_id.unwrap();

        let old_position = self.element(element_id, map, self.callback).position();
        let new_position = match self.affected_axis.as_ref().unwrap() {
            crate::gizmo::Axis::X => Coordinate {
                x: mouse_pos.x - self.cursor_to_element_offset.x,
                y: old_position.y,
            },
            crate::gizmo::Axis::Y => Coordinate {
                x: old_position.x,
                y: mouse_pos.y - self.cursor_to_element_offset.y,
            },
            crate::gizmo::Axis::XY => mouse_pos,
        };

        self.element_mut(element_id, map, self.callback_mut)
            .set_position(new_position);

        let a = map.intersections().clone();
        let keys = a.keys();
        for k in keys {
            map.update_intersection(&k);
        }
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, _button: u32, _map: &mut Map) {
        if self.active {
            self.active = false;
        }
    }

    fn render(
        &self,
        context: &web_sys::CanvasRenderingContext2d,
        camera: &Camera,
        map: &Map,
    ) -> Result<(), JsValue> {
        if let Some(selected_id) = self.element_id {
            let position = self.element(selected_id, map, self.callback).position();
            context.translate(position.x + camera.x as f64, position.y + camera.y as f64)?;

            self.x_axis.render(&self.x_style, &context)?;
            self.x_arrow.render(&self.x_style, &context)?;
            self.y_axis.render(&self.y_style, &context)?;
            self.y_arrow.render(&self.y_style, &context)?;

            context.set_transform(1., 0., 0., 1., 0., 0.)?;
        }

        Ok(())
    }

    fn element<F>(&'a self, id: Uuid, map: &'a Map, callback: F) -> &T
    where
        F: Fn(Uuid, &'a Map) -> &'a T,
    {
        callback(id, map)
    }

    fn element_mut<F>(&'a self, id: Uuid, map: &'a mut Map, callback: F) -> &mut T
    where
        F: Fn(Uuid, &'a mut Map) -> &'a mut T,
    {
        callback(id, map)
    }
}

/*
impl<Intersection> Gizmo<Intersection> for MoveGizmo {
    fn element(&self, id: Uuid, map: &mut Map) -> &Intersection {
        map.intersection(&id).unwrap()
    }
}
*/

static ARROW_WIDTH: f64 = 6.0;
static ARROW_HEIGHT: f64 = 10.0;
static LINE_LENGTH: f64 = 100.0;

pub struct MoveGizmo<T> {
    x_style: Style,
    x_axis: Line<f64>,
    x_arrow: Triangle<f64>,

    y_style: Style,
    y_axis: Line<f64>,
    y_arrow: Triangle<f64>,

    cursor_to_element_offset: Coordinate<f64>,

    pub element_id: Option<Uuid>,

    callback: fn(id: Uuid, map: &Map) -> &T,
    callback_mut: fn(id: Uuid, map: &mut Map) -> &mut T,

    active: bool,

    pub affected_axis: Option<Axis>,
}

impl<T> MoveGizmo<T> {
    pub fn new(
        callback: fn(id: Uuid, map: &Map) -> &T,
        callback_mut: fn(id: Uuid, map: &mut Map) -> &mut T,
    ) -> Self {
        MoveGizmo {
            callback,
            callback_mut,
            element_id: None,
            affected_axis: None,
            active: false,

            cursor_to_element_offset: Coordinate { x: 0., y: 0. },

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

        if diff.x.abs() < 30. && diff.y < 0. && diff.y >= -LINE_LENGTH {
            self.affected_axis = Some(Axis::Y);
            return;
        } 
        
        if diff.y.abs() < 30. && diff.x > 0. && diff.x <= LINE_LENGTH {
            self.affected_axis = Some(Axis::X);
            return;
        }

        if diff.x.abs() < 30. && diff.y.abs() < 30. {
            self.affected_axis = Some(Axis::XY);
            return;
        }
    }
}
