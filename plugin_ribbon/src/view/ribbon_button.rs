use std::ops::Deref;
use std::rc::Rc;
use std::{borrow::Borrow, cell::RefCell};

use yew::{function_component, html, use_state, Callback, Html, Properties};

use rust_editor::ui::tooltip::{Tooltip, TooltipPosition};

use crate::model::ribbon_button::RibbonButtonState;

#[derive(Properties)]
pub struct RibbonButtonProps {
    pub state: Rc<RefCell<RibbonButtonState>>,
    pub tooltip: Option<String>,
    pub icon: &'static str,
    pub on_click_callback: Rc<dyn Fn(Rc<RefCell<RibbonButtonState>>)>,
}

impl PartialEq for RibbonButtonProps {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state && self.tooltip == other.tooltip && self.icon == other.icon
    }
}

#[function_component]
pub fn RibbonButton(props: &RibbonButtonProps) -> Html {
    let state = props.state.clone();

    let onclick_callback = props.on_click_callback.clone();
    let onclick = {
        Callback::from(move |_| {
            let on_click_callback = onclick_callback.as_ref().borrow();
            on_click_callback(state.clone());
        })
    };

    let show_tooltip_handle = use_state(|| false);
    let show_tooltip = show_tooltip_handle.deref().clone();

    let show_tooltip_handle1 = show_tooltip_handle.clone();
    let onmouseover = { Callback::from(move |_| show_tooltip_handle1.set(true)) };

    let show_tooltip_handle2 = show_tooltip_handle.clone();
    let onmouseleave = { Callback::from(move |_| show_tooltip_handle2.set(false)) };

    let state_class = match *props.state.as_ref().borrow() {
        RibbonButtonState::Enabled => "",
        RibbonButtonState::Disabled => "disabled",
        RibbonButtonState::Selected => "selected",
        RibbonButtonState::Warning => "warning",
        RibbonButtonState::Error => "error",
    };

    html! {
        <button class={state_class} onclick={onclick} onmouseover={onmouseover} onmouseleave={onmouseleave} >
            <span class="material-icons">{&props.icon}</span>
            {
                html! {
                    if show_tooltip {
                        if let Some(tooltip) = &props.tooltip {
                            <Tooltip text={tooltip.clone()} position={TooltipPosition::Below} />
                        }
                    }
                }
            }
        </button>
    }
}
