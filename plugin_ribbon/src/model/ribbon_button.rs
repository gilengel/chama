use std::{cell::RefCell, rc::Rc};

use rust_editor::{ui::app::{App, EditorMessages}, log};
use yew::{html, Context, classes, use_state};

use super::ribbon_action::{ClickableRibbonAction, RibbonAction};

#[derive(PartialEq)]
pub enum RibbonButtonState {
    Enabled,
    Disabled,
    Selected,
    Warning,
    Error,
}
pub struct RibbonButton<Data>
where
    Data: Default,
{
    pub icon: &'static str,
    pub identifier: &'static str,
    pub tooltip: String,
    pub state: Rc<RefCell<RibbonButtonState>>,
    pub on_click_callback: Rc<dyn Fn(Rc<RefCell<RibbonButtonState>>) -> EditorMessages<Data>>,
}

impl<Data> RibbonButton<Data>
where
    Data: Default,
{
    pub fn new<T>(
        identifier: &'static str,
        icon: &'static str,
        tooltip: String,
        on_click_callback: T,
    ) -> Self
    where
        T: Fn(Rc<RefCell<RibbonButtonState>>) -> EditorMessages<Data> + 'static,
    {
        RibbonButton {
            identifier,
            icon,
            tooltip,
            state: Rc::new(RefCell::new(RibbonButtonState::Enabled)),
            on_click_callback: Rc::new(on_click_callback),
        }
    }
}

impl<Data> RibbonAction<Data> for RibbonButton<Data>
where
    Data: Default,
{
    fn view(&self, ctx: &Context<App<Data>>) -> yew::virtual_dom::VNode {
        let state = self.state.clone();
        let callback = Rc::clone(&self.on_click_callback);
        let onclick = ctx.link().callback(move |_| (*callback)(state.clone()));
        
        let state_class = match *self.state.borrow() {
            RibbonButtonState::Enabled => "",
            RibbonButtonState::Disabled => "disabled",
            RibbonButtonState::Selected => "selected",
            RibbonButtonState::Warning => "warning",
            RibbonButtonState::Error => "error",
        };

        html! {
            <button onclick={onclick} class={state_class} >
                <span class="material-icons">{&self.icon}</span>
                <span class="tooltip">{&self.tooltip}</span>
            </button>
        }
    }
}

impl<Data> ClickableRibbonAction<Data> for RibbonButton<Data>
where
    Data: Default,
{
    fn on_click_callback(
        &self,
        _: std::rc::Rc<dyn Fn() -> rust_editor::ui::app::EditorMessages<Data>>,
    ) {
    }
}
