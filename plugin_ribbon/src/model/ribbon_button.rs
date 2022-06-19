use std::{cell::RefCell, rc::Rc};

use rust_editor::ui::app::App;
use yew::{html, Context};

use super::ribbon_action::{ClickableRibbonAction, RibbonAction};

#[derive(PartialEq)]
pub enum RibbonButtonState {
    Enabled,
    Disabled,
    Selected,
    Warning,
    Error,
}
pub struct RibbonButton {
    pub icon: &'static str,
    pub identifier: &'static str,
    pub tooltip: Option<String>,
    pub state: Rc<RefCell<RibbonButtonState>>,
    pub on_click_callback: Rc<dyn Fn(Rc<RefCell<RibbonButtonState>>)>,
}

impl RibbonButton {
    pub fn new<T>(
        identifier: &'static str,
        icon: &'static str,
        tooltip: Option<String>,
        on_click_callback: T,
    ) -> Self
    where
        T: Fn(Rc<RefCell<RibbonButtonState>>) + 'static,
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

impl<Data> RibbonAction<Data> for RibbonButton
where
    Data: Default,
{
    fn view(&self, _: &Context<App<Data>>) -> yew::virtual_dom::VNode {
        use crate::view::ribbon_button::RibbonButton as UiRibbonButton;
        html! {
            <UiRibbonButton
                state={self.state.clone()}
                tooltip={self.tooltip.clone()}
                icon={self.icon}
                on_click_callback={self.on_click_callback.clone()}
            />
        }
    }
}

impl<Data> ClickableRibbonAction<Data> for RibbonButton
where
    Data: Default,
{
    fn on_click_callback(
        &self,
        _: std::rc::Rc<dyn Fn() -> rust_editor::ui::app::EditorMessages<Data>>,
    ) {
    }
}
