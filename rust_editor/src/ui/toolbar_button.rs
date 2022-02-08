use yew::{function_component, html, Callback, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct ButtonProps<S>
where
    S: Clone + PartialEq,
{
    pub icon: &'static str,
    pub tooltip: &'static str,

    /// The data transferred to the parent component each time the button is pressed. This
    /// is usually only a simple identifier such an enum value but you can send also structs
    /// or more complicated data with it. Keep in mind that sending large data structures can
    /// have a negative impact on runtime
    pub identifier: S,

   pub on_click: Callback<S>
}

#[function_component]
pub fn ToolbarButton<S>(props: &ButtonProps<S>) -> Html
where
    S: Clone + PartialEq + 'static,
{
    let on_click = props.on_click.clone();
    let identifier = props.identifier.clone();

    let onclick = Callback::from(move |_| { on_click.emit(identifier.clone()) });
    html! {
        <li>
        <button onclick={onclick}>
          <span class="material-icons">{props.icon}</span>

        </button>
        <span class="tooltip">{props.tooltip}</span>
      </li>
    }
}
