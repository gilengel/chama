use std::{cell::RefCell, f64::consts::PI, rc::Rc};

use geo::Coordinate;
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::street::Street;


#[derive(Clone)]
pub struct Intersection {
    pub id: Uuid,
    pub position: Coordinate<f64>,

    connected_streets: Vec<Rc<RefCell<Street>>>,
}

impl Intersection {
    pub fn new(
        position: Coordinate<f64>,
        connected_streets: Vec<Rc<RefCell<Street>>>,
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

    pub fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        context.begin_path();
        context.arc(self.position.x, self.position.y, 15.0, 0.0, 2.0 * PI)?;
        context.set_fill_style(&"#FF8C00".into());
        context.fill();

        /*
        context.set_fill_style(&"#000000".into());

        
        context.fill_text(
            &format!("c={}", self.connected_streets.len()).to_string(),
            self.position.x,
            self.position.y - 20.0,
        )?;

        let mut y = self.position.y;
        for street in &self.connected_streets {
            let street = street.as_ref().borrow();
            context.fill_text(&format!("{}", street.id).to_string(), self.position.x, y)?;

            y += 16.0;
        }
        */

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

    pub fn is_connected_to_street(&self, street: Rc<RefCell<Street>>) -> bool {
        self.connected_streets
            .iter()
            .any(|e| Rc::ptr_eq(&e,&street))
    }

    pub fn add_connected_street(&mut self, street: Rc<RefCell<Street>>) {
        self.connected_streets.push(street);
    }

    pub fn get_connected_streets(&self) -> &Vec<Rc<RefCell<Street>>> {
        &self.connected_streets
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
