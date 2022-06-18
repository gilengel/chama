use std::rc::Rc;

use rust_editor::ui::app::{EditorMessages, App};
use yew::{virtual_dom::VNode, Context};

pub trait RibbonAction<Data> {
  fn view(&self, ctx: &Context<App<Data>>) -> VNode where Data: Default;
}

pub trait ClickableRibbonAction<Data> : RibbonAction<Data> {
  fn on_click_callback(&self, callback: Rc<dyn Fn() -> EditorMessages<Data>>);
}