use yew::{function_component, html, Children, Properties, Html};

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