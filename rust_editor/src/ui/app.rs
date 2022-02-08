use std::collections::HashMap;

use yew::{html, AppHandle, Component, Context, Html, Properties};
use crate::log;
use crate::ui::toolbar_button::ToolbarButton;

use std::hash::Hash;

use crate::ui::toolbar::Toolbar;
use crate::{
    plugins::{camera::Renderer, plugin::Plugin},
    system::System,
};

pub enum EditorMessages<S, T>
where
    S: Clone + std::cmp::PartialEq,
{
    AddPlugin(Box<dyn Plugin<T>>),
    AddMode((S, Vec<Box<dyn System<T>>>, Option<ModeProps>)),
    SwitchMode(S),
}

pub struct App<S, T>
where
    T: Default + 'static,
    S: Clone + std::cmp::PartialEq,
{
    data: T,

    modes: HashMap<S, (Vec<Box<dyn System<T>>>, Option<ModeProps>)>,

    plugins: Vec<Box<dyn Plugin<T>>>,
}

#[derive(Properties, PartialEq, Default)]
pub struct EditorProps
//where
//    S: PartialEq + Eq + Hash + 'static,
{
    //#[prop_or_default]
//pub toolbars: DynChildren,
}

impl<S, T> Component for App<S, T>
where
    T: Default + 'static,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    type Message = EditorMessages<S, T>;
    type Properties = EditorProps; //<S>;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        App {
            data: T::default(),
            plugins: Vec::new(),
            modes: HashMap::new()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {


        match msg {
            EditorMessages::AddPlugin(mut e) => {
                let plugin = e.as_mut();
                //plugin.
                self.plugins.push(e);
            }
            EditorMessages::AddMode((index, mode, button)) => {
                self.modes.insert(index, (mode, button));
            }
            EditorMessages::SwitchMode(e) => log!("SWITCH MODE :)"),
        }

        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        html! {
        <main>
            <Toolbar>
            { for self.modes.iter().filter(|(_, (_, button))| button.is_some()).map(|(id, (_, button))| {
                let id = id.clone();
                let button = button.as_ref().unwrap();
                let on_click = ctx.link().callback(|e| EditorMessages::SwitchMode(e));

                html!{
                    
                    <ToolbarButton<S> icon={button.icon} tooltip={button.tooltip} identifier={id} {on_click} />
                    
                }
            })
        }
            </Toolbar>
        </main>
        }
    }
}

pub struct GenericEditor<S, T>
where
    T: Default + 'static,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    app_handle: AppHandle<App<S, T>>,
}

pub struct ModeProps {
    pub icon: &'static str,
    pub tooltip: &'static str,
}

impl<S, T> GenericEditor<S, T>
where
    T: Renderer + Default + 'static,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    pub fn add_plugin<P>(&mut self)
    where
        P: Plugin<T> + Default + 'static,
    {
        self.app_handle
            .send_message(EditorMessages::AddPlugin(Box::new(P::default())));
    }

    pub fn add_mode(
        &mut self,
        index: S,
        systems: Vec<Box<dyn System<T>>>,
        button: Option<ModeProps>,
    ) {
        self.app_handle
            .send_message(EditorMessages::AddMode((index, systems, button)));
    }
}

pub fn x_launch<S, T>() -> GenericEditor<S, T>
where
    T: Default + 'static,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    GenericEditor {
        app_handle: yew::start_app::<App<S, T>>(),
    }
}
