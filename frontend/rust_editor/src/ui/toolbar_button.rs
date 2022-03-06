use std::rc::Rc;

use super::app::EditorMessages;

pub struct ToolbarButton<Data> {
  pub icon: &'static str,
  pub identifier: &'static str,
  pub tooltip: String,
  pub on_click_callback: Rc<dyn Fn() -> EditorMessages<Data>>,
  pub selected: Option<Box<dyn Fn() -> bool>>,
}