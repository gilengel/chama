use yew::prelude::*;


use yew::Properties;

#[derive(PartialEq, Clone)]
pub enum TooltipPosition {
    Above,
    Below,
    Left,
    Right
}

impl From<TooltipPosition> for &'static str {
    fn from(position: TooltipPosition) -> Self {
        match position {
            TooltipPosition::Above => "above",
            TooltipPosition::Below => "below",
            TooltipPosition::Left => "left",
            TooltipPosition::Right => "right",
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct TooltipProps {
    pub text: String,
    pub position: TooltipPosition
}

#[function_component]
pub fn Tooltip(props: &TooltipProps) -> Html {
    let tooltip_ref = use_node_ref();

    let position: &'static str = props.position.clone().into();
    html! {
        <span ref={tooltip_ref} class={classes!("tooltip", position)}>{&props.text}</span>
    }
}
