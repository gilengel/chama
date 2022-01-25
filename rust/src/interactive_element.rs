use serde::{Serialize, Deserialize};

use crate::{style::Style};

#[derive(Clone, Serialize, Deserialize)]
pub enum InteractiveElementState {
    Normal,
    Hover,
    Selected
}

pub trait InteractiveElement {
    fn state(&self) -> InteractiveElementState {
        InteractiveElementState::Normal
    }

    fn set_state(&mut self, new_state: InteractiveElementState);

    fn style(&self) -> &Style;
}