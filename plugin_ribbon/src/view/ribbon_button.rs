use std::cell::RefCell;
use std::rc::Rc;

use rust_editor::ui::app::EditorMessages;
use yew::{Callback, Properties};

use crate::model::ribbon_button::{RibbonButtonState};

#[derive(Properties)]
pub struct RibbonButtonProps<Data> {
    pub state: Rc<RefCell<RibbonButtonState>>,
    pub tooltip: Option<String>,
    pub icon: &'static str,
    pub on_click: Callback<EditorMessages<Data>>, //pub on_click_callback: Rc<dyn Fn(Rc<RefCell<RibbonButtonState>>)>,
}

impl<Data> PartialEq for RibbonButtonProps<Data> {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state && self.tooltip == other.tooltip && self.icon == other.icon
    }
}

pub struct RibbonButton;

/*
impl Component for RibbonButton {
    type Message = ();
    type Properties = RibbonButtonProps<Data>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = ctx.props.state.clone();

        let onclick_callback = ctx.props.on_click.clone();
        let onclick = {
            Callback::from(move |_| {
                let on_click_callback = onclick_callback.as_ref().borrow();
                on_click_callback(state.clone());

                //ctx.props.on_click.emit(value)
            })
        };

        let show_tooltip_handle = use_state(|| false);
        let show_tooltip = show_tooltip_handle.deref().clone();

        let show_tooltip_handle1 = show_tooltip_handle.clone();
        let onmouseover = { Callback::from(move |_| show_tooltip_handle1.set(true)) };

        let show_tooltip_handle2 = show_tooltip_handle.clone();
        let onmouseleave = { Callback::from(move |_| show_tooltip_handle2.set(false)) };

        let state_class = match *ctx.props.state.as_ref().borrow() {
            RibbonButtonState::Enabled => "",
            RibbonButtonState::Disabled => "disabled",
            RibbonButtonState::Selected => "selected",
            RibbonButtonState::Warning => "warning",
            RibbonButtonState::Error => "error",
        };

        html! {
            <button class={state_class} onclick={onclick} onmouseover={onmouseover} onmouseleave={onmouseleave} >
                <span class="material-icons">{&ctx.props.icon}</span>
                {
                    html! {
                        if show_tooltip {
                            if let Some(tooltip) = &ctx.props.tooltip {
                                <Tooltip text={tooltip.clone()} position={TooltipPosition::Below} />
                            }
                        }
                    }
                }
            </button>
        }
    }
}
*/