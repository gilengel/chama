use serde::{Serialize, Deserialize};

use crate::{style::Style};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum InteractiveElementSystem {
    Normal,
    Hover,
    Selected
}

pub trait InteractiveElement {
    fn state(&self) -> InteractiveElementSystem;

    fn set_state(&mut self, new_state: InteractiveElementSystem);

    fn style(&self) -> &Style;
}