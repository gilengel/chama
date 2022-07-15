use std::{cell::RefCell, rc::Rc};

use rust_editor::ui::app::{App, EditorMessages};
use yew::{html, Context};

use super::ribbon_action::{ClickableRibbonAction, RibbonAction};

pub enum RibbonButtonType {
    Normal,
    Toggle,
}

impl Default for RibbonButtonType {
    fn default() -> Self {
        RibbonButtonType::Normal
    }
}

#[derive(PartialEq)]
pub enum RibbonButtonState {
    Enabled,
    Disabled,
    Selected,
}
pub struct RibbonButton<Data> {
    pub icon: &'static str,
    pub identifier: &'static str,
    pub tooltip: Option<String>,
    pub state: Rc<RefCell<RibbonButtonState>>,
    pub button_type: RibbonButtonType,
    pub on_click_callback: Rc<dyn Fn() -> EditorMessages<Data>>,
}

impl<Data> RibbonButton<Data>
where
    Data: Default,
{
    pub fn new(
        identifier: &'static str,
        icon: &'static str,
        tooltip: Option<String>,
        button_type: Option<RibbonButtonType>,
        on_click_callback: impl Fn() -> EditorMessages<Data> + 'static,
    ) -> Self {
        RibbonButton {
            identifier,
            icon,
            tooltip,
            state: Rc::new(RefCell::new(RibbonButtonState::Enabled)),
            button_type: button_type.unwrap_or_default(),
            on_click_callback: Rc::new(on_click_callback),
        }
    }

    pub fn set_state(&mut self, new_state: RibbonButtonState) {
        *self.state.borrow_mut() = new_state;
    }
}

impl<Data> RibbonAction<Data> for RibbonButton<Data>
where
    Data: Default,
{
    fn view(&self, ctx: &Context<App<Data>>) -> yew::virtual_dom::VNode {
        let callback = Rc::clone(&self.on_click_callback);
        let onclick = ctx.link().callback(move |_| (*callback)());
        /*
        let onclick = {
            Callback::from(move |_| {
                let on_click_callback = onclick_callback.as_ref().borrow();
                //on_click_callback(state.clone());

                //ctx.props.on_click.emit(value)
            })
        };
        */

        /*
               let show_tooltip_handle = use_state(|| false);
               let show_tooltip = show_tooltip_handle.deref().clone();

               let show_tooltip_handle1 = show_tooltip_handle.clone();
               let onmouseover = { Callback::from(move |_| show_tooltip_handle1.set(true)) };

               let show_tooltip_handle2 = show_tooltip_handle.clone();
               let onmouseleave = { Callback::from(move |_| show_tooltip_handle2.set(false)) };
        */
        let state_class = match *self.state.as_ref().borrow() {
            RibbonButtonState::Enabled => "",
            RibbonButtonState::Disabled => "disabled",
            RibbonButtonState::Selected => "selected",
        };

        html! {
            <button class={state_class} onclick={onclick} /*onmouseover={onmouseover} onmouseleave={onmouseleave}*/ >
                <span class="material-icons">{&self.icon}</span>
                {
                    html! {
                        /*
                        if show_tooltip {
                            if let Some(tooltip) = &self.tooltip {
                                <Tooltip text={tooltip.clone()} position={TooltipPosition::Below} />
                            }
                        }
                        */
                    }
                }
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
