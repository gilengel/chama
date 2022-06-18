use yew::{function_component, html, Html};

use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct RibbonButtonProps
{
    pub icon: String
}

#[function_component]
pub fn RibbonButton(props: &RibbonButtonProps) -> Html
{

    html! {
        <button>
            <span class="material-icons">{&props.icon}</span>
        </button>
    }
}
