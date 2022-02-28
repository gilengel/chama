use core::fmt;

use yew::{function_component, html, Children, Html, Properties};

#[derive(Hash, PartialEq, Eq, Clone)]
pub enum ToolbarPosition {
    Left,
    Right,
    Top,
    Bottom,
}

impl fmt::Display for ToolbarPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ToolbarPosition::Left => write!(f, "left_primary_toolbar"),
            ToolbarPosition::Right => write!(f, "right_primary_toolbar"),
            ToolbarPosition::Top => write!(f, "top_primary_toolbar"),
            ToolbarPosition::Bottom => write!(f, "bottom_primary_toolbar"),
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct ToolbarProps {
    pub children: Children,
}

#[function_component]
pub fn Toolbar(props: &ToolbarProps) -> Html {
    html! {
        <ul class="toolbar">
            { for props.children.iter() }
        </ul>
    }
}
