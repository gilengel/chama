use gloo_render::{request_animation_frame, AnimationFrame};
use rust_internal::PluginExecutionBehaviour;
use std::any::Any;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use thiserror::Error;
use wasm_bindgen::JsCast;
use yew::html::Scope;

use crate::plugins::camera::Camera;
use crate::plugins::plugin::{PluginWithOptions, SpecialKey};

use crate::error;

use crate::ui::dialog::Dialog;
use geo::Coordinate;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent, MouseEvent};
use yew::{html, AppHandle, Component, Context, Html, NodeRef, Properties};

use super::toolbar::{Toolbar, ToolbarPosition, Toolbars};

#[macro_export]
macro_rules! keys {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

pub enum EditorMessages<Data> {
    AddPlugin(
        (
            &'static str,
            Rc<RefCell<dyn PluginWithOptions<Data> + 'static>>,
        ),
    ),
    AddPlugins(
        Vec<(
            &'static str,
            Rc<RefCell<dyn PluginWithOptions<Data> + 'static>>,
        )>,
    ),
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

pub type Plugins<Data> = BTreeMap<PluginId, Rc<RefCell<dyn PluginWithOptions<Data>>>>;
pub type PluginsVec<Data> = HashMap<PluginId, Rc<RefCell<dyn PluginWithOptions<Data>>>>;

pub struct App<Data>
where
    Data: Default + 'static,
{
    /// Holds the displayed data
    data: Data,

    /// All plugins that implement the editor logic and functionality
    plugins: Plugins<Data>,

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

    canvas_size: Coordinate<i32>,
}

impl<Data> App<Data>
where
    Data: Default + 'static,
{
    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Data {
        &mut self.data
    }

    pub fn plugin<'a, Plugin, F>(&self, mut f: F)
    where
        Plugin: PluginWithOptions<Data> + Default + 'static,
        F: FnMut(&Plugin),
    {
        if let Some(plugin) = self.plugins.get(Plugin::identifier()) {
            let plugin = plugin.as_ref().borrow_mut();
            let plugin = plugin.as_any().downcast_ref::<Plugin>().unwrap();

            f(plugin);
        }
    }

    pub fn plugin_mut<'a, Plugin>(&mut self, f: fn(&Plugin))
    where
        Plugin: PluginWithOptions<Data> + 'static,
    {
        if let Some(plugin) = self.plugins.get_mut(Plugin::identifier()) {
            let plugin = plugin.as_ref().borrow_mut();
            let plugin = plugin.as_any().downcast_ref::<Plugin>().unwrap();

            f(plugin);
        }
    }

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
    Data: Default + 'static,
{
    type Message = EditorMessages<Data>;
    type Properties = EditorProps;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        App {
            data: Data::default(),
            plugins: BTreeMap::new(),
            shortkeys: HashMap::new(),
            toolbars: Toolbars::new(),

            canvas_ref: NodeRef::default(),
            _render_loop: None,
            context: None,

            pressed_keys: Vec::new(),
            canvas_size: Coordinate { x: 1920, y: 1080 },
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

            self.canvas_size = Coordinate {
                x: canvas.offset_width(),
                y: canvas.offset_height(),
            };

            // A reference to the handle must be stored, otherwise it is dropped and the render won't
            // occur.
            self._render_loop = Some(handle);
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EditorMessages::AddPlugin((key, plugin)) => {
                if let Err(e) = plugin.as_ref().borrow_mut().startup(self) {
                    error!("{}", e)
                }
                self.plugins.insert(key, plugin);

                ctx.link().send_message(EditorMessages::ActivatePlugin(key))
            }
            EditorMessages::AddPlugins(plugins) => {
                //self.plugins.reserve(plugins.len());

                for (key, plugin) in plugins {
                    if let Err(e) = plugin.as_ref().borrow_mut().startup(self) {
                        error!("{}", e)
                    }

                    self.plugins.insert(key, plugin);

                    ctx.link().send_message(EditorMessages::ActivatePlugin(key))
                }
            }
            EditorMessages::MouseMove(e) => {
                let mouse_pos = self.mouse_pos(e.client_x() as u32, e.client_y() as u32);
                let mouse_diff = Coordinate {
                    x: e.movement_x() as f64,
                    y: e.movement_y() as f64,
                };

                for (_, plugin) in enabled_plugins(&mut self.plugins) {
                    plugin
                        .as_ref()
                        .borrow_mut()
                        .mouse_move(mouse_pos, mouse_diff, self);
                }
            }
            EditorMessages::MouseDown(e) => {
                let mouse_pos = self.mouse_pos(e.client_x() as u32, e.client_y() as u32);

                let enabled_plugins = enabled_plugins(&mut self.plugins);
                for (_, plugin) in &enabled_plugins {
                    plugin
                        .as_ref()
                        .borrow_mut()
                        .mouse_down(mouse_pos, e.button() as u32, self);
                }
            }
            EditorMessages::MouseUp(e) => {
                let mouse_pos = self.mouse_pos(e.client_x() as u32, e.client_y() as u32);

                for (_, plugin) in enabled_plugins(&mut self.plugins) {
                    plugin
                        .as_ref()
                        .borrow_mut()
                        .mouse_up(mouse_pos, e.button() as u32, self);
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
                                .as_ref()
                                .borrow_mut()
                                .shortkey_pressed(shortkey, ctx);
                        }
                    }
                }

                for (_, plugin) in enabled_plugins(&mut self.plugins) {
                    plugin.as_ref().borrow_mut().key_down(&e.key()[..], self);
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

                for (_, plugin) in enabled_plugins(&mut self.plugins) {
                    plugin.as_ref().borrow_mut().key_up(&e.key()[..], self);
                }
            }
            EditorMessages::Render(_) => {
                self.render(ctx.link());
            }
            EditorMessages::PluginOptionUpdated((plugin, attribute, value)) => {
                let plugin = self.plugins.get(plugin).unwrap_or_else(|| panic!("plugin with key {} is not present but received an option update. Make sure that the plugin is not destroyed during runtime", plugin));
                plugin
                    .as_ref()
                    .borrow_mut()
                    .update_property(attribute, value);
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
                        let x = x.borrow();
                        x.enabled()
                            && x.execution_behaviour() == &PluginExecutionBehaviour::Exclusive
                    })
                {
                    exclusive_active_plugin.as_ref().borrow_mut().disable();
                }

                self.plugins
                    .get_mut(plugin_id)
                    .unwrap()
                    .as_ref()
                    .borrow_mut()
                    .enable();
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

            /*
            TODO reenable options for plugins
            <Dialog>
            {
                for self.plugins.iter().map(|(_, plugin)| {
                    plugin.borrow().view_options(ctx)
                })
            }
            </Dialog>
            */

            {
                self.toolbars.view(ctx)
            }
        </main>
        }
    }
}

fn enabled_plugins<Data>(plugins: &Plugins<Data>) -> PluginsVec<Data>
where
    Data: Default + 'static,
{
    plugins
        .iter()
        .filter(|(_, plugin)| plugin.borrow().enabled())
        .map(|(id, plugin)| (*id, Rc::clone(plugin)))
        .collect()
}

/*
pub fn plugin<P, Data>(plugins: &PluginsVec<Data>) -> Option<&P> where P: PluginWithOptions<Data> + 'static, Data: Default + 'static {
    if let Some(plugin) = plugins.get(P::identifier()) {
        return Some(plugins.get(P::identifier()).unwrap().borrow().as_any().downcast_ref::<P>().unwrap())
    }

    None
}
*/

impl<Data> App<Data>
where
    Data: Default + 'static,
{
    fn mouse_pos(&self, x: u32, y: u32) -> Coordinate<f64> {
        let mut offset: Coordinate<f64> = Coordinate { x: 0., y: 0. };
            
        self.plugin(|camera: &Camera| {            
            offset = Coordinate { x: camera.x(), y: camera.y() };
        });    

        return Coordinate {
            x: x as f64 - offset.x,
            y: y as f64 - offset.y,
        };
    }

    pub fn render(&mut self, link: &Scope<Self>) {
        let context = self.context.as_ref().unwrap();

        context.set_transform(1., 0., 0., 1., 0., 0.).unwrap();
        

        context.clear_rect(
            0.0,
            0.0,
            self.canvas_size.x.into(),
            self.canvas_size.y.into(),
        );

        self.plugin(|camera: &Camera| {            
            context.translate(camera.x(), camera.y()).unwrap();
        });

        for (_, plugin) in enabled_plugins(&mut self.plugins) {
            plugin.as_ref().borrow_mut().render(context, self);
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
    Data: Default + 'static,
{
    app_handle: AppHandle<App<Data>>,
}

pub struct ModeProps {
    pub icon: &'static str,
    pub tooltip: &'static str,
}

impl<Data> GenericEditor<Data>
where
    Data: Default + 'static,
{
    pub fn add_plugin<P>(&mut self, plugin: P)
    where
        P: PluginWithOptions<Data> + 'static,
    {
        self.app_handle.send_message(EditorMessages::AddPlugin((
            P::identifier(),
            Rc::new(RefCell::new(plugin)),
        )));
    }
}

pub fn x_launch<Data>() -> GenericEditor<Data>
where
    Data: Default + 'static,
{
    GenericEditor {
        app_handle: yew::start_app::<App<Data>>(),
    }
}
