use std::{cell::RefCell, collections::HashMap, panic, rc::Rc};

use geo::Coordinate;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::plugins::camera::{Camera, Renderer};
use crate::plugins::plugin::Plugin;

use crate::plugins::toolbar::ToolbarPlugin;
use crate::toolbar::{Toolbar, ToolbarPosition, ToolbarRadioButton};
use crate::{system::System, InformationLayer};

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

pub fn slugify(text: &str) -> String {
    let a: String = text
        .chars()
        .map(|x| match x {
            '!' | '?' | ',' | '.' | ';' | ' ' => '-',
            _ => x,
        })
        .collect();

    a.to_ascii_lowercase()
}

pub fn add_mode<T>(
    editor: Rc<RefCell<Editor<T>>>,
    mode: u8,
    systems: Vec<Box<dyn System<T> + Send + Sync>>,
    icon: &'static str,
    tooltip: &'static str,
) where
    T: Renderer + Default + 'static,
{
    if editor.borrow().modes.contains_key(&mode) {
        // TODO Error handling
        todo!()
    }

    editor
        .as_ref()
        .borrow_mut()
        .modes
        .insert(mode as u8, systems);

    if let Some(toolbar_plugin) = get_plugin::<T, ToolbarPlugin<T>>(editor.borrow().plugins()) {
        let button = Box::new(ToolbarRadioButton::new(
            icon,
            tooltip,
            0,
            |_| {
                //e.clone().borrow_mut().switch_mode(mode.clone())
            },
        ));

        let toolbar = match toolbar_plugin
            .toolbars()
            .iter_mut()
            .find(|x| x.id == "modes.toolbar")
        {
            Some(e) => { e.add_button(button, editor.clone()); },
            None => {
                let toolbar =
                    Toolbar::<T>::new(vec![button], ToolbarPosition::Top, "modes.toolbar".to_string());

                //toolbar_plugin.toolbars.get_mut(&ToolbarPosition::Top).unwrap().push(toolbar);    
                toolbar.render(editor.clone());            
            }
        };
    }
}

pub fn get_plugin<T, S>(plugins: &Vec<Box<dyn Plugin<T>>>) -> Option<&S>
where
    S: 'static,
    T: Renderer + 'static,
{
    for plugin in plugins {
        if let Some(p) = plugin.as_ref().as_any().downcast_ref::<S>() {
            return Some(p);
        }
    }

    None
}

pub fn get_plugin_mut<T, S>(plugins: &mut Vec<Box<dyn Plugin<T>>>) -> Option<&mut S>
where
    S: 'static,
    T: Renderer + 'static,
{
    for plugin in plugins {
        if let Some(p) = plugin.as_any_mut().downcast_mut::<S>() {
            return Some(p);
        }
    }

    None
}

pub struct Editor<T>
where
    T: Renderer + 'static,
{
    pub context: Option<CanvasRenderingContext2d>,
    additional_information_layers: Vec<InformationLayer>,

    data: T,

    plugins: Vec<Box<dyn Plugin<T>>>,

    active_mode: u8,
    modes: HashMap<u8, Vec<Box<dyn System<T> + Send + Sync + 'static>>>,
}

pub fn add_plugin<T, S>(editor: Rc<RefCell<Editor<T>>>, mut plugin: S)
where
    S: Plugin<T> + 'static,
    T: Renderer + Default + 'static,
{
    plugin.on_startup(editor.clone());

    let additional_toolbars = plugin.toolbars();

    if !additional_toolbars.is_empty() {
        let mut e = editor.as_ref().borrow_mut();

        if let Some(toolbar_plugin) = get_plugin_mut::<T, ToolbarPlugin<T>>(e.plugins_mut()) {
            let toolbars = toolbar_plugin
                .toolbars
                .get_mut(&ToolbarPosition::Top)
                .unwrap();

            for additional_toolbar in additional_toolbars {
                // In case the toolbar already exists, add the additional buttons to it
                if let Some(toolbar) = toolbars
                    .iter_mut()
                    .find(|toolbar| toolbar.id == additional_toolbar.id)
                {
                    for additional_button in additional_toolbar.buttons {
                        toolbar
                            .add_button(additional_button, editor.clone())
                            .expect("Could not add button to existing toolbar");
                    }

                    continue;
                }

                // Add it to the DOM
                additional_toolbar
                    .render(editor.clone())
                    .expect("could not add toolbar");

                // Add the new toolbar
                toolbars.push(additional_toolbar);
            }
        }
    }

    editor.as_ref().borrow_mut().plugins.push(Box::new(plugin));
}

impl<T> Editor<T>
where
    T: Renderer + Default,
{
    pub fn new(_width: u32, _height: u32) -> Editor<T> {
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        Editor {
            context: None,
            additional_information_layers: vec![],
            active_mode: 0,
            data: T::default(),
            plugins: Vec::new(),
            modes: HashMap::new(),
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn plugins(&self) -> &Vec<Box<dyn Plugin<T>>> {
        &self.plugins
    }

    pub fn plugins_mut(&mut self) -> &mut Vec<Box<dyn Plugin<T>>> {
        &mut self.plugins
    }

    pub fn get_plugin<S>(&self) -> Option<&S>
    where
        S: 'static,
    {
        get_plugin::<T, S>(&self.plugins)
    }

    pub fn execute_plugin<S>(&mut self)
    where
        S: Plugin<T> + 'static,
    {
        let position = self
            .plugins
            .iter_mut()
            .position(|plugin| plugin.as_any_mut().downcast_mut::<S>().is_some());

        if position.is_none() {
            return;
        }

        let mut plugin = self.plugins.remove(position.unwrap());
        plugin.execute(self);

        self.plugins.insert(position.unwrap(), plugin);
    }

    pub fn get_plugin_mut<S>(&mut self) -> Option<&mut S>
    where
        S: 'static,
    {
        get_plugin_mut::<T, S>(&mut self.plugins)
    }

    pub fn switch_mode(&mut self, new_active_mode: u8) {
        self.active_mode = new_active_mode;
    }

    /*
    pub fn add_toolbar(&mut self, toolbar: Toolbar<T>) {
        if !self.toolbars.contains_key(&toolbar.position) {
            self.toolbars.insert(toolbar.position, vec![]);
        }

        let position = toolbar.position.clone();
        self.toolbars.get_mut(&position).unwrap().push(toolbar);
    }
    */

    pub fn render(&self) -> Result<(), JsValue> {
        let context = self.context.as_ref().unwrap();

        context.clear_rect(0.0, 0.0, 2000.0, 2000.0);

        for system in self.modes.get(&self.active_mode).unwrap().iter() {
            system.render(
                &self.data,
                &context,
                &self.additional_information_layers,
                &self.plugins,
            )?;

            if system.blocks_next_systems() {
                break;
            }
        }

        Ok(())
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

    pub fn mouse_down(&mut self, x: u32, y: u32, button: u32) {
        let mouse_pos = self.mouse_pos(x, y);

        for plugin in &mut self.plugins {
            plugin.mouse_down(mouse_pos, button, &mut self.data);
        }

        let active_mode = self.modes.get_mut(&self.active_mode).unwrap();
        for system in active_mode.iter_mut() {
            system.mouse_down(mouse_pos, button, &mut self.data, &mut self.plugins);

            if system.blocks_next_systems() {
                break;
            }
        }
    }

    pub fn mouse_up(&mut self, x: u32, y: u32, button: u32) {
        let mouse_pos = self.mouse_pos(x, y);

        for plugin in &mut self.plugins {
            plugin.mouse_up(mouse_pos, button, &mut self.data);
        }

        for system in self.modes.get_mut(&self.active_mode).unwrap().iter_mut() {
            system.mouse_up(mouse_pos, button, &mut self.data, &mut self.plugins);

            if system.blocks_next_systems() {
                break;
            }
        }
    }

    pub fn mouse_move(&mut self, x: u32, y: u32, dx: i32, dy: i32) {
        let mouse_pos = self.mouse_pos(x, y);
        let mouse_diff = Coordinate {
            x: dx as f64,
            y: dy as f64,
        };

        for plugin in &mut self.plugins {
            plugin.mouse_move(mouse_pos, mouse_diff, &mut self.data);
        }

        for system in self.modes.get_mut(&self.active_mode).unwrap().iter_mut() {
            system.mouse_move(mouse_pos, &mut self.data, &mut self.plugins);

            if system.blocks_next_systems() {
                break;
            }
        }
    }
}
