use gloo_render::{request_animation_frame, AnimationFrame};
use rust_internal::PluginExecutionBehaviour;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use thiserror::Error;
use wasm_bindgen::JsCast;
use yew::html::Scope;

use crate::plugins::camera::Camera;
use crate::plugins::plugin::{PluginWithOptions, SpecialKey};

use crate::{error, InformationLayer, log};

use crate::ui::dialog::Dialog;
use geo::Coordinate;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent, MouseEvent};
use yew::{html, AppHandle, Callback, Component, Context, Html, NodeRef, Properties, classes};

use crate::plugins::camera::Renderer;

use super::toolbar::ToolbarPosition;

#[macro_export]
macro_rules! keys {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

pub enum EditorMessages<Data> {
    AddPlugin((&'static str, Box<dyn PluginWithOptions<Data> + 'static>)),
    PluginOptionUpdated((&'static str, &'static str, Box<dyn Any>)),
    ActivatePlugin(&'static str),

    MouseMove(MouseEvent),
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    Render(f64),
}

pub type Shortkey = Vec<String>;

pub type PluginId = &'static str;

#[derive(Error, Debug)]
pub enum EditorError {
    #[error("shortkey {:?} is already registered", shortkey)]
    ShortkeyExists { shortkey: Shortkey },

    #[error("toolbar with id {:?} is not registered. Make sure to add it first to the editor before adding buttons to it.", id)]
    ToolbarDoesNotExists { id: &'static str },

    #[error("toolbar with id {:?} is already registered.", id)]
    ToolbarExists { id: &'static str },
}

pub struct Toolbar<Data> {
    pub id: &'static str,
    pub buttons: Vec<ToolbarButton<Data>>,
}

impl<Data> Toolbar<Data> {
    pub fn add_button(
        &mut self,
        icon: &'static str,
        identifier: &'static str,
        tooltip: String,
        on_click_callback: impl Fn() -> EditorMessages<Data> + 'static,
    ) -> Result<(), EditorError> {
        let btn = ToolbarButton {
            icon,
            identifier,
            tooltip,
            on_click_callback: Rc::new(on_click_callback),
            selected: None
        };

        self.buttons.push(btn);

        Ok(())
    }

    pub fn add_toggle_button(
        &mut self,
        icon: &'static str,
        identifier: &'static str,
        tooltip: String,
        toggled: impl Fn() -> bool + 'static,
        on_click_callback: impl Fn() -> EditorMessages<Data> + 'static,
    ) -> Result<(), EditorError> {
        let btn = ToolbarButton {
            icon,
            identifier,
            tooltip,
            on_click_callback: Rc::new(on_click_callback),
            selected: Some(Box::new(toggled))
        };

        self.buttons.push(btn);

        Ok(())
    }
}

struct Toolbars<Data> {
    toolbars: HashMap<ToolbarPosition, Vec<Toolbar<Data>>>,
}

impl<Data> Toolbars<Data>
where
    Data: Renderer + Default,
{
    pub fn new() -> Self {
        Toolbars {
            toolbars: HashMap::new(),
        }
    }

    fn view_button(&self, button: &ToolbarButton<Data>, ctx: &Context<App<Data>>) -> Html {
        let mut classes = classes!();
        if let Some(selected_callback) = &button.selected {
            if selected_callback() {
                classes.push("selected");
            }
        }

        let callback = Rc::clone(&button.on_click_callback);
        let onclick = ctx.link().callback(move |_| (*callback)() );
        html! {
            <li>
            <button onclick={onclick} class={classes}>
              <span class="material-icons">{button.icon}</span>

            </button>
            <span class="tooltip">{&button.tooltip}</span>
          </li>
        }
    }
    fn view_toolbar(&self, toolbar: &Toolbar<Data>, ctx: &Context<App<Data>>) -> Html {
        html! {
            <ul class="toolbar">
                {
                    for toolbar.buttons.iter().map(|button| {
                        self.view_button(button, ctx)
                    })
                }
            </ul>
        }
    }
    fn view_toolbars_at_pos(&self, pos: &ToolbarPosition, ctx: &Context<App<Data>>) -> Html {
        if !self.toolbars.contains_key(pos) {
            return html! {};
        }

        let id = pos.to_string();

        html! {
            <div id={id}>
            {
                for self.toolbars.get(pos).unwrap().iter().map(|toolbar| self.view_toolbar(toolbar, ctx))
            }
            </div>
        }
    }
    pub fn view(&self, ctx: &Context<App<Data>>) -> Html {
        let positions = vec![
            ToolbarPosition::Left,
            ToolbarPosition::Right,
            ToolbarPosition::Top,
            ToolbarPosition::Bottom,
        ];

        html! {
            for positions.iter().map(|pos| self.view_toolbars_at_pos(pos, ctx))
        }
    }

    pub fn add_toolbar<'a>(
        &'a mut self,
        toolbar_id: &'static str,
        position: ToolbarPosition,
    ) -> Result<&'a mut Toolbar<Data>, EditorError> {
        for (_, toolbars) in &mut self.toolbars {
            if let Some(_) = toolbars.iter_mut().find(|toolbar| toolbar.id == toolbar_id) {
                return Err(EditorError::ToolbarExists { id: toolbar_id });
            }
        }

        let toolbar = Toolbar {
            id: toolbar_id,
            buttons: vec![],
        };

        if !self.toolbars.contains_key(&position) {
            self.toolbars.insert(position.clone(), vec![toolbar]);
        } else {
            self.toolbars.get_mut(&position).unwrap().push(toolbar);
        }

        let toolbar = self
            .toolbars
            .get_mut(&position)
            .unwrap()
            .last_mut()
            .unwrap();

        Ok(toolbar)
    }

    pub fn index_and_position_of_toolbar(
        &self,
        id: &'static str,
    ) -> Result<(ToolbarPosition, usize), EditorError> {
        let mut result = self
            .toolbars
            .iter()
            .map(|(pos, toolbars)| {
                (
                    (*pos).clone(),
                    toolbars.iter().position(|toolbar| toolbar.id == id),
                )
            })
            .collect::<Vec<(ToolbarPosition, Option<usize>)>>();
        result.retain(|(_, x)| x.is_some());

        if result.is_empty() {
            return Err(EditorError::ToolbarDoesNotExists { id });
        }

        let (pos, index) = result.first().unwrap();
        let result = ((*pos).clone(), index.unwrap());
        Ok(result)
    }
}
pub struct ToolbarButton<Data> {
    pub icon: &'static str,
    pub identifier: &'static str,
    pub tooltip: String,
    pub on_click_callback: Rc<dyn Fn() -> EditorMessages<Data>>,
    pub selected: Option<Box<dyn Fn() -> bool>>
}

pub struct App<Data>
where
    Data: Renderer + Default + 'static,
{
    /// Holds the displayed data
    data: Data,

    /// TODO not needed right now - should be moved into a plugin
    _additional_information_layers: Vec<InformationLayer>,

    /// All plugins that implement the editor logic and functionality
    plugins: HashMap<PluginId, Box<dyn PluginWithOptions<Data>>>,

    /// Toolbars added by plugins
    toolbars: Toolbars<Data>,

    /// Registered by plugins, shortkeys must by unique.
    shortkeys: HashMap<PluginId, Vec<Shortkey>>,

    /// Black magic needed by yew
    _render_loop: Option<AnimationFrame>,
    canvas_ref: NodeRef,
    context: Option<CanvasRenderingContext2d>,

    /// Internally stores the pressed keys as registered by native web events.
    /// Keys are pushed to the end so the vec is sorted from oldest pressed key to newest
    pressed_keys: Vec<String>,
}

impl<Data> App<Data>
where
    Data: Renderer + Default + 'static,
{
    pub fn add_shortkey<T>(&mut self, keys: Shortkey) -> Result<(), EditorError>
    where
        T: PluginWithOptions<Data>,
    {
        match self.shortkeys.values().find(|x| x.contains(&&keys)) {
            Some(_) => Err(EditorError::ShortkeyExists { shortkey: keys }),
            None => {
                let id = T::identifier();
                if !self.shortkeys.contains_key(id) {
                    self.shortkeys.insert(id, vec![]);
                };

                self.shortkeys.get_mut(id).unwrap().push(keys);
                Ok(())
            }
        }
    }

    pub fn add_toolbar(
        &mut self,
        toolbar_id: &'static str,
        position: ToolbarPosition,
    ) -> Result<&mut Toolbar<Data>, EditorError> {
        self.toolbars.add_toolbar(toolbar_id, position)
    }

    pub fn get_or_add_toolbar(
        &mut self,
        toolbar_id: &'static str,
        position: ToolbarPosition,
    ) -> Result<&mut Toolbar<Data>, EditorError> {
        match self.toolbars.index_and_position_of_toolbar(toolbar_id) {
            Ok((pos, index)) => Ok(self
                .toolbars
                .toolbars
                .get_mut(&pos)
                .unwrap()
                .get_mut(index)
                .unwrap()),

            Err(_) => return self.toolbars.add_toolbar(toolbar_id, position),
        }
    }
}

#[derive(Properties, PartialEq, Default)]
pub struct EditorProps {}

impl<Data> Component for App<Data>
where
    Data: Renderer + Default + 'static,
{
    type Message = EditorMessages<Data>;
    type Properties = EditorProps;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        App {
            data: Data::default(),
            plugins: HashMap::new(),
            shortkeys: HashMap::new(),
            toolbars: Toolbars::new(),
            _additional_information_layers: Vec::new(),

            canvas_ref: NodeRef::default(),
            _render_loop: None,
            context: None,

            pressed_keys: Vec::new(),
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
            EditorMessages::AddPlugin((key, mut plugin)) => {
                
                if let Err(e) = plugin.startup(self) {
                    error!("{}", e)
                }
                self.plugins.insert(key, plugin);
                

                ctx.link().send_message(EditorMessages::ActivatePlugin(key))
            }
            EditorMessages::MouseMove(e) => {
                let mouse_pos = self.mouse_pos(e.client_x() as u32, e.client_y() as u32);
                let mouse_diff = Coordinate {
                    x: e.movement_x() as f64,
                    y: e.movement_y() as f64,
                };

                for plugin in self
                    .plugins
                    .values_mut()
                    .into_iter()
                    .filter(|plugin| plugin.enabled())
                {
                    plugin.mouse_move(mouse_pos, mouse_diff, &mut self.data);
                }
            }
            EditorMessages::MouseDown(e) => {
                let mouse_pos = self.mouse_pos(e.client_x() as u32, e.client_y() as u32);

                for plugin in self
                    .plugins
                    .values_mut()
                    .into_iter()
                    .filter(|plugin| plugin.enabled())
                {
                    plugin.mouse_down(mouse_pos, e.button() as u32, &mut self.data);
                }
            }
            EditorMessages::MouseUp(e) => {
                let mouse_pos = self.mouse_pos(e.client_x() as u32, e.client_y() as u32);

                for plugin in self
                    .plugins
                    .values_mut()
                    .into_iter()
                    .filter(|plugin| plugin.enabled())
                {
                    plugin.mouse_up(mouse_pos, e.button() as u32, &mut self.data);
                }
            }
            EditorMessages::KeyDown(e) => {
                e.prevent_default();

                match self.pressed_keys.last() {
                    Some(last) => {
                        if *last != e.key() {
                            self.pressed_keys.push(e.key())
                        }
                    }
                    None => self.pressed_keys.push(e.key()),
                }

                for (plugin_id, shortkeys) in self.shortkeys.iter_mut() {
                    for shortkey in shortkeys {
                        if self.pressed_keys.ends_with(shortkey) {
                            self.plugins
                                .get_mut(plugin_id)
                                .unwrap()
                                .shortkey_pressed(shortkey, ctx);
                        }
                    }
                }

                for plugin in self
                    .plugins
                    .values_mut()
                    .into_iter()
                    .filter(|plugin| plugin.enabled())
                {
                    plugin.key_down(&e.key()[..], &mut self.data);
                }
            }
            EditorMessages::KeyUp(e) => {
                self.pressed_keys.retain(|value| *value != e.key());

                let mut special_keys = vec![];
                if e.ctrl_key() {
                    special_keys.push(SpecialKey::Ctrl);
                }
                if e.alt_key() {
                    special_keys.push(SpecialKey::Alt);
                }
                if e.shift_key() {
                    special_keys.push(SpecialKey::Shift);
                }

                for plugin in self
                    .plugins
                    .values_mut()
                    .into_iter()
                    .filter(|plugin| plugin.enabled())
                {
                    plugin.key_up(&e.key()[..], &mut self.data);
                }
            }
            EditorMessages::Render(_) => {
                self.render(ctx.link());
            }
            EditorMessages::PluginOptionUpdated((plugin, attribute, value)) => {
                let plugin = self.get_plugin_by_key_mut(plugin).unwrap_or_else(|| panic!("plugin with key {} is not present but received an option update. Make sure that the plugin is not destroyed during runtime", plugin));
                plugin.update_property(attribute, value);
            }
            EditorMessages::ActivatePlugin(plugin_id) => {
                if !self.plugins.contains_key(plugin_id) {
                    error!(
                        "tried to activate plugin with id {} which is not registered",
                        plugin_id
                    );
                    return true;
                }

                if let Some((_, exclusive_active_plugin)) =
                    self.plugins.iter_mut().find(|(_, x)| {
                        x.enabled()
                            && x.execution_behaviour() == &PluginExecutionBehaviour::Exclusive
                    })
                {
                    exclusive_active_plugin.disable();
                }

                self.plugins.get_mut(plugin_id).unwrap().enable();
            }
        }

        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let onmousedown = ctx.link().callback(|e| EditorMessages::MouseDown(e));
        let onmouseup = ctx.link().callback(|e| EditorMessages::MouseUp(e));
        let onmousemove = ctx.link().callback(|e| EditorMessages::MouseMove(e));

        let onkeyup = ctx.link().callback(|e| EditorMessages::KeyUp(e));
        let onkeydown = ctx.link().callback(|e| EditorMessages::KeyDown(e));

        html! {
        <main>
            <canvas ref={self.canvas_ref.clone()} width="2560" height="1440" {onmousedown} {onmouseup} {onmousemove} {onkeyup} {onkeydown} tabindex="0"></canvas>

            <Dialog>
            {
                for self.plugins.iter().map(|(_, plugin)| {
                    plugin.view_options(ctx)
                })
            }
            </Dialog>

            {
                self.toolbars.view(ctx)
            }

            /*
            <Toolbar>
            {

                for self.modes.iter().filter(|(_, (_, button))| button.is_some()).map(|(id, (_, button))| {
                    html!{
                        self.view_mode_button(ctx, id, button)
                    }
                })

            }
            </Toolbar>
            */
        </main>
        }
    }
}

impl<Data> App<Data>
where
    Data: Renderer + Default + 'static,
{
    fn get_plugin_by_key_mut(&mut self, key: &str) -> Option<&mut dyn PluginWithOptions<Data>>
    where
        Data: Renderer + 'static,
    {
        if let Some(plugin) = self.plugins.get_mut(key) {
            return Some(&mut **plugin);
        }

        None
    }

    pub fn get_plugin<P>(&self) -> Option<&P>
    where
        P: PluginWithOptions<Data> + 'static,
        Data: Renderer + 'static,
    {
        for (_, plugin) in &self.plugins {
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

        self.data.render(context, &vec![]).unwrap();

        for plugin in self
            .plugins
            .values()
            .into_iter()
            .filter(|plugin| plugin.enabled())
        {
            plugin.render(context);
        }

        let handle = {
            let link = link.clone();
            request_animation_frame(move |time| link.send_message(EditorMessages::Render(time)))
        };

        // A reference to the new handle must be retained for the next render to run.
        self._render_loop = Some(handle);
    }
}

pub struct GenericEditor<Data>
where
    Data: Renderer + Default + 'static,
{
    app_handle: AppHandle<App<Data>>,
}

pub struct ModeProps {
    pub icon: &'static str,
    pub tooltip: &'static str,
}

impl<Data> GenericEditor<Data>
where
    Data: Renderer + Default + 'static,
{
    pub fn add_plugin<P>(&mut self, plugin: P)
    where
        P: PluginWithOptions<Data> + 'static,
    {
        self.app_handle.send_message(EditorMessages::AddPlugin((
            P::identifier(),
            Box::new(plugin),
        )));
    }
}

pub fn x_launch<Data>() -> GenericEditor<Data>
where
    Data: Renderer + Default + 'static,
{
    GenericEditor {
        app_handle: yew::start_app::<App<Data>>(),
    }
}

pub fn add_plugin<Data, P>(editor: &mut GenericEditor<Data>, plugin: P)
where
Data: Renderer + Default + 'static,
P: PluginWithOptions<Data> + 'static {
    
}