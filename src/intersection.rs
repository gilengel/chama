use std::{rc::Rc, cell::RefCell, f64::consts::PI};

use geo::Coordinate;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::street::Street;

#[derive(Clone)]
pub struct Intersection {
    pub position: Coordinate<f64>,

    pub connected_streets: Vec<Rc<RefCell<Street>>>,
}

impl Intersection {
    pub fn set_position(&mut self, position: Coordinate<f64>) {
        self.position = position;
    }

    pub fn get_position(&self) -> Coordinate<f64> {
        self.position
    }

    pub fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        context.begin_path();
        context.arc(self.position.x, self.position.y, 15.0, 0.0, 2.0 * PI)?;
        context.set_fill_style(&"#FF8C00".into());
        context.fill();

        context.set_fill_style(&"#000000".into());

        let mut y = self.position.y;
        for street in &self.connected_streets {
            let street = street.as_ref().borrow();
            context.fill_text(
                &format!("{}", street.id).to_string(),
                self.position.x,
                y,
            )?;

            y += 16.0;
        }

        Ok(())
    }

    pub fn remove_connected_street(&mut self, street: Rc<RefCell<Street>>) {
        if let Some(index) = self
            .connected_streets
            .iter()
            .position(|i| Rc::ptr_eq(&i, &street))
        {
            self.connected_streets.remove(index);
        }
    }
}

impl Default for Intersection {
    fn default() -> Self {
        Intersection {
            position: Coordinate { x: 0., y: 0. },
            connected_streets: vec![],
        }
    }
}