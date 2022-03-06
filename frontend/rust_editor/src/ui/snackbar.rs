use std::fmt;

use yew::{function_component, html, Html, Properties, classes};

#[derive(Clone, PartialEq)]
pub enum SnackbarPosition {
    Left,
    Center,
    Right
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

#[derive(Properties, PartialEq)]
pub struct SnackbarProps {
    #[prop_or_else(default_snackbar_position)]
    pub position: SnackbarPosition,

    pub message: String
}

#[function_component]
pub fn Snackbar(props: &SnackbarProps) -> Html {
    html! {
        <div class={classes!("md-snackbar", props.position.to_string())}>
            {props.message.clone()}
        </div>
    }
}