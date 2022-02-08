use gloo_render::{request_animation_frame, AnimationFrame};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use yew::html::Scope;

use crate::plugins::camera::Camera;
use crate::ui::toolbar_button::ToolbarButton;
use crate::InformationLayer;
use geo::Coordinate;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent};
use yew::{html, AppHandle, Component, Context, Html, NodeRef, Properties};

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

    MouseMove(MouseEvent),
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    Render(f64),
}

pub struct App<S, T>
where
    T: Default + 'static,
    S: Clone + std::cmp::PartialEq,
{
    data: T,

    additional_information_layers: Vec<InformationLayer>,
    active_mode: Option<S>,
    modes: HashMap<S, (Vec<Box<dyn System<T>>>, Option<ModeProps>)>,

    plugins: Vec<Box<dyn Plugin<T>>>,
    _render_loop: Option<AnimationFrame>,
    canvas_ref: NodeRef,
    context: Option<CanvasRenderingContext2d>,
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
    T: Renderer + Default + 'static,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    type Message = EditorMessages<S, T>;
    type Properties = EditorProps; //<S>;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        App {
            data: T::default(),
            plugins: Vec::new(),
            modes: HashMap::new(),
            active_mode: None,
            additional_information_layers: Vec::new(),

            canvas_ref: NodeRef::default(),
            _render_loop: None,
            context: None,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let canvas = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();

        self.context = Some(
            canvas
                .get_context("2d")
                .expect("Canvas context expected")
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .expect("Converting into context 2d failed"),
        );

        if first_render {
            let handle = {
                let link = ctx.link().clone();
                request_animation_frame(move |time| link.send_message(EditorMessages::Render(time)))
            };

            // A reference to the handle must be stored, otherwise it is dropped and the render won't
            // occur.
            self._render_loop = Some(handle);
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EditorMessages::AddPlugin(e) => {
                self.plugins.push(e);
            }
            EditorMessages::AddMode((index, mode, button)) => {
                self.modes.insert(index, (mode, button));
            }
            EditorMessages::SwitchMode(e) => self.active_mode = Some(e),
            EditorMessages::MouseMove(e) => {
                if self.active_mode == None {
                    return false;
                }

                let mouse_pos = self.mouse_pos(e.client_x() as u32, e.client_y() as u32);
                let mouse_diff = Coordinate {
                    x: e.movement_x() as f64,
                    y: e.movement_y() as f64,
                };

                for plugin in &mut self.plugins {
                    plugin.mouse_move(mouse_pos, mouse_diff, &mut self.data);
                }

                let (active_mode, _) = self
                    .modes
                    .get_mut(self.active_mode.as_ref().unwrap())
                    .unwrap();
                for system in active_mode.iter_mut() {
                    system.mouse_move(mouse_pos, &mut self.data, &mut self.plugins);

                    if system.blocks_next_systems() {
                        break;
                    }
                }
            }
            EditorMessages::MouseDown(e) => {
                if self.active_mode == None {
                    return false;
                }

                let mouse_pos = self.mouse_pos(e.client_x() as u32, e.client_y() as u32);

                for plugin in &mut self.plugins {
                    plugin.mouse_down(mouse_pos, e.button() as u32, &mut self.data);
                }

                let (active_mode, _) = self
                    .modes
                    .get_mut(&self.active_mode.as_ref().unwrap())
                    .unwrap();
                for system in active_mode.iter_mut() {
                    system.mouse_down(
                        mouse_pos,
                        e.button() as u32,
                        &mut self.data,
                        &mut self.plugins,
                    );

                    if system.blocks_next_systems() {
                        break;
                    }
                }
            }
            EditorMessages::MouseUp(e) => {
                if self.active_mode == None {
                    return false;
                }

                let mouse_pos = self.mouse_pos(e.client_x() as u32, e.client_y() as u32);

                for plugin in &mut self.plugins {
                    plugin.mouse_up(mouse_pos, e.button() as u32, &mut self.data);
                }

                let (active_mode, _) = self
                    .modes
                    .get_mut(&self.active_mode.as_ref().unwrap())
                    .unwrap();
                for system in active_mode.iter_mut() {
                    system.mouse_up(
                        mouse_pos,
                        e.button() as u32,
                        &mut self.data,
                        &mut self.plugins,
                    );

                    if system.blocks_next_systems() {
                        break;
                    }
                }
            }
            EditorMessages::Render(_) => {
                let context = self.context.as_ref().unwrap();
                for plugin in &mut self.plugins {
                    plugin.render(context);
                }

                self.render(ctx.link());
            }
        }

        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let onmousedown = ctx.link().callback(|e| EditorMessages::MouseDown(e));
        let onmouseup = ctx.link().callback(|e| EditorMessages::MouseUp(e));
        let onmousemove = ctx.link().callback(|e| EditorMessages::MouseMove(e));
        html! {
        <main>
            <canvas ref={self.canvas_ref.clone()} width="1920" height="1080" {onmousedown} {onmouseup} {onmousemove}></canvas>
            <Toolbar>
            { for self.modes.iter().filter(|(_, (_, button))| button.is_some()).map(|(id, (_, button))| {

                html!{
                    self.view_mode_button(ctx, id, button)
                }
            })
        }
            </Toolbar>
        </main>
        }
    }
}

impl<S, T> App<S, T>
where
    T: Renderer + Default + 'static,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    fn view_mode_button(
        &self,
        ctx: &yew::Context<Self>,
        id: &S,
        mode_props: &Option<ModeProps>,
    ) -> Html {
        let id = id.clone();
        let mode_props = mode_props.as_ref().unwrap();
        let on_click = ctx.link().callback(|e| EditorMessages::SwitchMode(e));

        html! {
            <ToolbarButton<S> icon={mode_props.icon} tooltip={mode_props.tooltip} identifier={id} {on_click} />
        }
    }

    pub fn get_plugin<P>(&self) -> Option<&P>
    where
        P: Plugin<T> + 'static,
        T: Renderer + 'static,
    {
        for plugin in &self.plugins {
            if let Some(p) = plugin.as_ref().as_any().downcast_ref::<P>() {
                return Some(p);
            }
        }

        None
    }

    fn mouse_pos(&self, x: u32, y: u32) -> Coordinate<f64> {
        let offset = match self.get_plugin::<Camera>() {
            Some(x) => Coordinate { x: x.x(), y: x.y() },
            None => Coordinate { x: 0., y: 0. },
        };

        return Coordinate {
            x: x as f64 - offset.x,
            y: y as f64 - offset.y,
        };
    }

    pub fn render(&mut self, link: &Scope<Self>) {
        let context = self.context.as_ref().unwrap();

        context.clear_rect(0.0, 0.0, 2000.0, 2000.0);

        for plugin in &mut self.plugins {
            plugin.render(context);
        }

        let (active_mode, _) = self
            .modes
            .get_mut(&self.active_mode.as_ref().unwrap())
            .unwrap();
        for system in active_mode.iter_mut() {
            system
                .render(
                    &self.data,
                    &context,
                    &self.additional_information_layers,
                    &self.plugins,
                )
                .expect("System could not render");

            if system.blocks_next_systems() {
                break;
            }
        }

        let handle = {
            let link = link.clone();
            request_animation_frame(move |time| link.send_message(EditorMessages::Render(time)))
        };

        // A reference to the new handle must be retained for the next render to run.
        self._render_loop = Some(handle);
    }
}

pub struct GenericEditor<S, T>
where
    T: Renderer + Default + 'static,
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

    pub fn activate_mode(&mut self, mode: S) {
        self.app_handle
            .send_message(EditorMessages::SwitchMode(mode));
    }
}

pub fn x_launch<S, T>() -> GenericEditor<S, T>
where
    T: Renderer + Default + 'static,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    GenericEditor {
        app_handle: yew::start_app::<App<S, T>>(),
    }
}
