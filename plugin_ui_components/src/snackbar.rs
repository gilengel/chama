use std::{borrow::Borrow, fmt, rc::Rc};

use gloo_timers::callback::Interval;
use yew::{
    classes, function_component, html, use_effect_with_deps, use_reducer, Callback, Html,
    Properties, Reducible,
};
#[derive(Default, Debug)]
struct SecondsState {
    seconds: usize,
}

enum SecondsStateAction {
    Increment,
}

impl Reducible for SecondsState {
    /// Reducer Action Type
    type Action = SecondsStateAction;

    /// Reducer Function
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            SecondsStateAction::Increment => Self {
                seconds: self.seconds + 1,
            }
            .into(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum SnackbarPosition {
    Left,
    Center,
    Right,
}

impl Default for SnackbarPosition {
    fn default() -> Self {
        SnackbarPosition::Left    
    }
}

impl fmt::Display for SnackbarPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SnackbarPosition::Left => write!(f, "left"),
            SnackbarPosition::Center => write!(f, "center"),
            SnackbarPosition::Right => write!(f, "right"),
        }
    }
}

fn default_snackbar_position() -> SnackbarPosition {
    SnackbarPosition::Left
}

pub struct SnackbarAction {
    pub label: String,
    pub callback: Rc<dyn Fn()>,
}

#[derive(Properties)]
pub struct SnackbarProps {
    #[prop_or_else(default_snackbar_position)]
    pub position: SnackbarPosition,
    pub message: String,
    pub action: Option<SnackbarAction>,
}

impl PartialEq for SnackbarProps {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.message == other.message
    }
}

#[function_component]
pub fn Snackbar(props: &SnackbarProps) -> Html {
    let seconds_state_handle = use_reducer(SecondsState::default);

    use_effect_with_deps(
        {
            let seconds_state_handle = seconds_state_handle.clone();

            move |_| {
                // i intervals get out of scope they get dropped and destroyed
                let interval = Interval::new(1000, move || {
                    seconds_state_handle.dispatch(SecondsStateAction::Increment)
                });

                // So we move it into the clean up function, rust will consider this still being used and wont drop it
                // then we just drop it ourselves in the cleanup
                move || drop(interval)
            }
        },
        (), // Only create the interval once per your component existence
    );

    if seconds_state_handle.seconds < 4 {
        html! {
            <div class={classes!("md-snackbar", props.position.to_string())}>
                {props.message.clone()}
                {
                    if let Some(action) = &props.action {
                        let onclick_callback = action.callback.clone();
                        let onclick = {
                            Callback::from(move |_| {
                                let on_click_callback = onclick_callback.as_ref().borrow();
                                on_click_callback();
                            })
                        };

                        html! {
                            <button onclick={onclick}>{"ACTION"}</button>
                        }
                    } else {
                        html! {
                            <> </>
                        }
                    }
                }

            </div>
        }
    }else {
        html!{
            <> </>
        }
    }
}
