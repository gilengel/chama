use gloo_render::{request_animation_frame, AnimationFrame};
use rust_internal::PluginExecutionBehaviour;
use std::any::Any;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use thiserror::Error;
use wasm_bindgen::JsCast;
use yew::html::Scope;

use crate::input::keyboard::Key;
//use crate::plugins::camera::Camera;
//use crate::plugins::plugin::{PluginWithOptions, SpecialKey};

use crate::error;
use crate::plugin::{PluginWithOptions, SpecialKey};
use geo::Coordinate;
use web_sys::{
    CanvasRenderingContext2d, DragEvent, HtmlCanvasElement, KeyboardEvent, MouseEvent, PointerEvent,
};

use yew::{html, AppHandle, Component, Context, Html, NodeRef, Properties};

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
    PluginMessage(&'static str, Box<dyn Any>),
    ActivatePlugin(&'static str),

    MouseMove(MouseEvent),
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    ShortkeyPressed(Shortkey),
    Render(f64),
    UpdateElements(),
    Drop(DragEvent),
    DragOver(DragEvent),
    RerenderView,
}

pub type Shortkey = Vec<Key>;

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

    /// Registered by plugins, shortkeys must by unique.
    shortkeys: HashMap<PluginId, Vec<Shortkey>>,

    /// Black magic needed by yew
    _render_loop: Option<AnimationFrame>,
    canvas_ref: NodeRef,
    context: Option<CanvasRenderingContext2d>,

    /// Internally stores the pressed keys as registered by native web events.
    /// Keys are pushed to the end so the vec is sorted from oldest pressed key to newest
    pressed_keys: Vec<Key>,

    canvas_size: Coordinate<i32>,

    last_mouse_pos: Coordinate<f64>,
}

// Not functional. Is used for test cases
impl<Data> Default for App<Data>
where
    Data: Default,
{
    fn default() -> Self {
        Self {
            data: Default::default(),
            plugins: Default::default(),
            shortkeys: Default::default(),
            _render_loop: Default::default(),
            canvas_ref: Default::default(),
            context: Default::default(),
            pressed_keys: Default::default(),
            canvas_size: Default::default(),
            last_mouse_pos: Coordinate { x: 0., y: 0. },
        }
    }
}

impl<Data> App<Data>
where
    Data: Default + 'static,
{
    /// Returns a non mutable reference to the data hold by the editor.
    pub fn data(&self) -> &Data {
        &self.data
    }

    /// Returns a mutable reference to the data hold by the editor.
    pub fn data_mut(&mut self) -> &mut Data {
        &mut self.data
    }

    /// Replaces the data hold by the editor by `data`.
    pub fn set_data(&mut self, data: Data) {
        self.data = data
    }

    /// Finds a plugin that was registered to the editor instance and let you perform non mutable actions on it.
    /// To perform the action you need to specify a closure `f`.
    ///
    /// # Example
    ///
    /// ```
    ///
    /// app.plugin(move |plugin: &DataPlugin<Map>| {
    ///     plugin.read();
    /// });
    ///
    /// ```
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

    /// Finds a plugin that was registered to the editor instance and let you perform mutable actions on it.
    /// To perform the action you need to specify a closure `f`.
    ///
    /// # Example
    ///
    /// ```
    ///
    /// app.plugin_mut(move |plugin: &mut DataPlugin<Map>| {
    ///     plugin.write("some_data");
    /// });
    ///
    /// ```
    pub fn plugin_mut<'a, Plugin, F>(&mut self, mut f: F)
    where
        Plugin: PluginWithOptions<Data> + Default + 'static,
        F: FnMut(&mut Plugin),
    {
        if let Some(plugin) = self.plugins.get_mut(Plugin::identifier()) {
            let mut plugin = plugin.as_ref().borrow_mut();
            let plugin = plugin.as_any_mut().downcast_mut::<Plugin>().unwrap();

            f(plugin);
        }
    }

    /// Finds two plugins that were registered to the editor instance and let you perform mutable actions on them simultanously.
    /// To perform the action you need to specify a closure `f`.
    ///
    /// # Example
    ///
    /// ```
    ///
    /// app.two_plugin_mut(move |plugin: &mut DataPlugin<Map>| {
    ///     plugin.write("some_data");
    /// });
    ///
    /// ```
    pub fn two_plugin_mut<'a, Plugin1, Plugin2, F>(&mut self, mut f: F)
    where
        Plugin1: PluginWithOptions<Data> + Default + 'static,
        Plugin2: PluginWithOptions<Data> + Default + 'static,
        F: FnMut(&mut Plugin1, &mut Plugin2),
    {
        let plugins: Vec<&Rc<RefCell<dyn PluginWithOptions<Data>>>> = self
            .plugins
            .iter()
            .filter(|(id, _)| id == &&Plugin1::identifier() || id == &&Plugin2::identifier())
            .map(|(_, plugin)| plugin)
            .collect();

        let mut plugin1 = plugins[0].as_ref().borrow_mut();
        let plugin1 = plugin1.as_any_mut().downcast_mut::<Plugin1>().unwrap();

        let mut plugin2 = plugins[1].as_ref().borrow_mut();
        let plugin2 = plugin2.as_any_mut().downcast_mut::<Plugin2>().unwrap();

        f(plugin1, plugin2)
    }

    /// Registers a shortkey that is handled by the editor.
    ///
    /// Shortkeys are usally used for plugins to allow fast execution of specific actions for expert users.
    /// Each plugin can register multiple shortkeys so that they can be mapped to different actions.
    /// Each plugin has a event handler that is triggered if a registered shortkey was processed by the app.
    ///
    /// # Errors
    ///
    /// An [ShortkeyExists](EditorError) error will be returned if the shortkey already exists.
    ///
    /// # Example
    ///
    /// ```
    ///
    /// impl<Data> Plugin<Data> for MyPlugin<Data>
    /// where
    ///     Data: Default + 'static,
    /// {
    ///     fn startup(&mut self, editor: &mut App<Data>) -> Result<(), EditorError> {
    ///         editor.add_shortkey::<MyPlugin<Data>>(vec![Key::Ctrl, Key::Z])?;
    ///
    ///         Ok(())
    ///     }
    ///
    ///     fn shortkey_pressed(&mut self, key: &Shortkey, _: &Context<App<Data>>, editor: &mut App<Data>) {
    ///         if *key == vec![Key::Ctrl, Key::Z] {
    ///             
    ///         }
    ///     }
    /// }
    ///
    /// ```
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

    /// Returns if a shortkey exists or not
    pub fn has_shortkey(&self, key: Shortkey) -> bool {
        self.shortkeys.values().any(|x| x.contains(&&key))
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

    fn create(_: &yew::Context<Self>) -> Self {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("should have a body");

        // TODO handle resize of window properly. Currently the canvas size is fixed to the initial window size

        App {
            data: Data::default(),
            plugins: BTreeMap::new(),
            shortkeys: HashMap::new(),
            canvas_ref: NodeRef::default(),
            _render_loop: None,
            context: None,

            pressed_keys: Vec::new(),
            canvas_size: Coordinate {
                x: body.client_width(),
                y: body.client_height(),
            },
            last_mouse_pos: Coordinate { x: 0., y: 0. },
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

            // A reference to the handle must be stored, otherwise it is dropped and the render won't occur.
            self._render_loop = Some(handle);
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EditorMessages::Drop(e) => {
                e.prevent_default();

                for (_, plugin) in enabled_plugins(&mut self.plugins) {
                    plugin.as_ref().borrow_mut().drop(e.clone())
                }
            }
            EditorMessages::DragOver(e) => {
                e.prevent_default();
            }
            EditorMessages::UpdateElements() => return true,
            EditorMessages::AddPlugin((key, plugin)) => {
                if let Err(e) = plugin.as_ref().borrow_mut().startup(self) {
                    error!("{}", e)
                }
                self.plugins.insert(key, plugin);

                ctx.link().send_message(EditorMessages::ActivatePlugin(key));
                return true;
            }
            EditorMessages::AddPlugins(plugins) => {
                for (key, plugin) in plugins {
                    if let Err(e) = plugin.as_ref().borrow_mut().startup(self) {
                        error!("{}", e)
                    }

                    self.plugins.insert(key, plugin);

                    ctx.link().send_message(EditorMessages::ActivatePlugin(key));
                }

                return true;
            }
            EditorMessages::MouseMove(e) => {
                let mouse_pos = self.mouse_pos(e.client_x() as u32, e.client_y() as u32);
                let mouse_diff = mouse_pos - self.last_mouse_pos;

                for (_, plugin) in enabled_plugins(&mut self.plugins) {
                    if plugin.as_ref().borrow_mut().mouse_move(
                        mouse_pos,
                        mouse_diff,
                        e.button().into(),
                        self,
                    ) {
                        break;
                    }
                }
            }
            EditorMessages::MouseDown(e) => {
                let mouse_pos = self.mouse_pos(e.client_x() as u32, e.client_y() as u32);

                self.last_mouse_pos = mouse_pos;

                let enabled_plugins = enabled_plugins(&mut self.plugins);
                for (_, plugin) in &enabled_plugins {
                    if plugin
                        .as_ref()
                        .borrow_mut()
                        .mouse_down(mouse_pos, e.button().into(), self)
                    {
                        break;
                    }
                }
            }
            EditorMessages::MouseUp(e) => {
                let mouse_pos = self.mouse_pos(e.client_x() as u32, e.client_y() as u32);

                self.last_mouse_pos = mouse_pos;

                for (_, plugin) in enabled_plugins(&mut self.plugins) {
                    if plugin
                        .as_ref()
                        .borrow_mut()
                        .mouse_up(mouse_pos, e.button().into(), self)
                    {
                        break;
                    }
                }
            }
            EditorMessages::KeyDown(e) => {
                e.prevent_default();

                let key: Key = e.key().into();
                match self.pressed_keys.last() {
                    Some(last) => {
                        if *last != key {
                            self.pressed_keys.push(key)
                        }
                    }
                    None => self.pressed_keys.push(key),
                }

                for shortkeys in self.shortkeys.values() {
                    for shortkey in shortkeys {
                        if self.pressed_keys.ends_with(shortkey) {
                            ctx.link()
                                .send_message(EditorMessages::ShortkeyPressed(shortkey.clone()));
                        }
                    }
                }

                for (_, plugin) in enabled_plugins(&mut self.plugins) {
                    plugin.as_ref().borrow_mut().key_down(e.key().into(), self);
                }

                return true;
            }
            EditorMessages::KeyUp(e) => {
                self.pressed_keys.retain(|value| *value != e.key().into());

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
                    plugin.as_ref().borrow_mut().key_up(e.key().into(), self);
                }

                return true;
            }
            EditorMessages::ShortkeyPressed(shortkey) => {
                for (plugin_id, shortkeys) in self.shortkeys.clone().iter() {
                    if shortkeys.contains(&shortkey) {
                        let plugin = Rc::clone(self.plugins.get(plugin_id).unwrap());
                        let mut plugin = plugin.as_ref().borrow_mut();
                        plugin.shortkey_pressed(&shortkey, ctx, self);
                    }
                }

                return true;
            }
            EditorMessages::Render(_) => {
                self.render(ctx.link());
            }
            EditorMessages::PluginOptionUpdated((plugin, attribute, value)) => {
                let plugin = Rc::clone(self.plugins.get(plugin).unwrap_or_else(|| panic!("plugin with key {} is not present but received an option update. Make sure that the plugin is not destroyed during runtime", plugin)));
                plugin
                    .as_ref()
                    .borrow_mut()
                    .update_property(attribute, value);

                plugin
                    .as_ref()
                    .borrow_mut()
                    .property_updated(attribute, self);

                return true;
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

                return true;
            }
            EditorMessages::PluginMessage(plugin_id, message) => {
                if !self.plugins.contains_key(plugin_id) {
                    error!(
                        "tried to send message to a plugin with id {} which is not registered",
                        plugin_id
                    );
                    return true;
                }

                self.plugins
                    .get_mut(plugin_id)
                    .unwrap()
                    .as_ref()
                    .borrow_mut()
                    .on_message(message);
            }
            EditorMessages::RerenderView => return true,
        }

        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        // Mouse events
        let onmousedown = ctx.link().callback(|e| EditorMessages::MouseDown(e));
        let onmouseup = ctx.link().callback(|e| EditorMessages::MouseUp(e));
        let onmousemove = ctx.link().callback(|e| EditorMessages::MouseMove(e));

        // Context menu event aka right click
        /*
        let oncontextmenu = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            EditorMessages::MouseUp(e)
        });
        */

        // Key events
        let onkeyup = ctx.link().callback(|e| EditorMessages::KeyUp(e));
        let onkeydown = ctx.link().callback(|e| EditorMessages::KeyDown(e));

        // Drag/Drop events
        let ondrop = ctx.link().callback(|e| EditorMessages::Drop(e));
        let ondragover = ctx.link().callback(|e| EditorMessages::DragOver(e));

        // Graphic tablet events
        let onpointermove = ctx
            .link()
            .callback(|_: PointerEvent| EditorMessages::Render(0.0));

        let enabled_plugins: Vec<Rc<RefCell<dyn PluginWithOptions<Data>>>> = self
            .plugins
            .iter()
            .filter(|(_, plugin)| plugin.borrow().enabled())
            .map(|(_, plugin)| Rc::clone(plugin))
            .collect();

        let mut plugin_elements: Vec<Html> = Vec::new();
        enabled_plugins.iter().for_each(|plugin| {
            plugin_elements.append(&mut plugin.borrow_mut().editor_elements(ctx, self));
        });

        html! {
            <main>
            {
                plugin_elements
            }
            <content>
                <canvas
                    ref={self.canvas_ref.clone()}
                    width={Some(self.canvas_size.x.to_string())}
                    height={Some(self.canvas_size.y.to_string())}
                    tabindex="0"

                    {ondrop}
                    {ondragover}
                    {onmousedown}
                    {onmouseup}
                    {onmousemove}
                    {onkeyup}
                    {onkeydown}
                    {onpointermove}
                    //{oncontextmenu}
                ></canvas>
            </content>
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

impl<Data> App<Data>
where
    Data: Default + 'static,
{
    fn mouse_pos(&self, x: u32, y: u32) -> Coordinate<f64> {
        let offset: Coordinate<f64> = Coordinate { x: 0., y: 0. };

        /*
        self.plugin(|camera: &Camera| {
            offset = Coordinate {
                x: camera.x(),
                y: camera.y(),
            };
        });
        */

        return Coordinate {
            x: x as f64 - offset.x,
            y: y as f64 - offset.y,
        };
    }

    pub fn plugins(
        &self,
    ) -> std::collections::btree_map::Iter<
        '_,
        &str,
        std::rc::Rc<RefCell<(dyn PluginWithOptions<Data> + 'static)>>,
    > {
        self.plugins.iter()
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

        /*
        self.plugin(|camera: &Camera| {
            context.translate(camera.x(), camera.y()).unwrap();
        });
        */

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
        app_handle: yew::Renderer::<App<Data>>::new().render(),
    }
}

pub fn launch<Data>(id: &str) -> GenericEditor<Data>
where
    Data: Default + 'static,
{
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let parent = document.get_element_by_id(id).unwrap();

    GenericEditor {
        app_handle: yew::Renderer::<App<Data>>::with_root(parent).render(),
    }
}
