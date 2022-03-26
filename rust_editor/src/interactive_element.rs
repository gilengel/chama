use serde::{Deserialize, Serialize};

use crate::style::Style;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum InteractiveElementState {
    Normal,
    Hover,
    Selected,
}

pub trait InteractiveElement {
    fn state(&self) -> InteractiveElementState;

    fn set_state(&mut self, new_state: InteractiveElementState);

    fn style(&self) -> &Style;
}
