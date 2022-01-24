use geo::{Coordinate, Line, Triangle};
use wasm_bindgen::JsValue;

pub enum Axis {
    X,
    Y,
    XY,
}

use crate::{
    map::Map,
    renderer::PrimitiveRenderer,
    style::Style,
    Camera,
};

pub trait GetPosition {
    fn position(&self) -> Coordinate<f64>;
}

pub trait SetPosition {
    fn set_position(&mut self, position: Coordinate<f64>);
}

pub trait Gizmo {
    fn mouse_over(&mut self, mouse_pos: Coordinate<f64>, origin: Coordinate<f64>) -> bool;
    fn mouse_down(&mut self, mouse_pos: Coordinate<f64>, button: u32, map: &mut Map);
    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map);
    fn mouse_up(&mut self, mouse_pos: Coordinate<f64>, button: u32, map: &mut Map);
    fn render(
        &self,
        context: &web_sys::CanvasRenderingContext2d,
        camera: &Camera,
    ) -> Result<(), JsValue>;
}

impl Gizmo for MoveGizmo {
    fn mouse_over(&mut self, mouse_pos: Coordinate<f64>, origin: Coordinate<f64>) -> bool {
        let diff = mouse_pos - origin;

        self.affected_axis(mouse_pos, origin);

        if !(diff.x >= 0. && diff.x <= LINE_LENGTH && diff.y <= 0. && diff.y >= -LINE_LENGTH) {
            return false;
        }

        true
    }

    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, _button: u32, _map: &mut Map) {
        
    }

    fn mouse_move(&mut self, _mouse_pos: Coordinate<f64>, _map: &mut Map) {
        todo!()
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, _button: u32, _map: &mut Map) {
        todo!()
    }

    fn render(
        &self,
        context: &web_sys::CanvasRenderingContext2d,
        camera: &Camera,
    ) -> Result<(), JsValue> {
        context.translate(camera.x.into(), camera.y.into());

        self.x_axis.render(&self.x_style, &context)?;
        self.x_arrow.render(&self.x_style, &context)?;
        self.y_axis.render(&self.y_style, &context)?;
        self.y_arrow.render(&self.y_style, &context)?;

        Ok(())
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

pub struct MoveGizmo {
    x_style: Style,
    x_axis: Line<f64>,
    x_arrow: Triangle<f64>,

    y_style: Style,
    y_axis: Line<f64>,
    y_arrow: Triangle<f64>,

    pub affected_axis: Option<Axis>,
}

impl MoveGizmo {
    pub fn new() -> Self {

        MoveGizmo {
            affected_axis: None,

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
        }

        if diff.y.abs() < 30. && diff.x > 0. && diff.x <= LINE_LENGTH {
            self.affected_axis = Some(Axis::X);
        }

        if diff.x.abs() < 30. && diff.y.abs() < 30. {
            self.affected_axis = Some(Axis::XY);
        }
    }
}
