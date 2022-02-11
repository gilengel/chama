#![allow(warnings)]

use std::any::Any;
use geo::Coordinate;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

pub trait GetPluginWithOptions<T> {
    fn get<S>(&self) -> Option<S>;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

pub struct Attribute {
    pub label: String,
    pub description: String,
}
pub struct NumberAttribute {
    pub default: i128,
    pub min: i128,
    pub max: i128,
}

pub struct BoolAttribute {
    pub default: bool
}

pub struct TextAttribute {
    pub default: String
}

pub enum PluginAttributes {
    Number((Attribute, NumberAttribute)),
    Text((Attribute, TextAttribute)),
    Bool((Attribute, BoolAttribute)),
}

pub struct Plugins<T> {
    count: usize,
    render_plugins: Vec<Box<dyn PluginRenderer>>,
    mouse_events_plugins: Vec<Box<dyn PluginMouseEvents<T>>>,
    keyboard_events_plugins: Vec<Box<dyn PluginKeyboardEvents>>,
}

impl<T> Plugins<T> {
    pub fn borrow_component_vec<ComponentType: 'static>(&self) -> Option<&Vec<Option<ComponentType>>> {

        None
    }

    
}

pub trait Renderer {
    fn render(
        &self,
        context: &CanvasRenderingContext2d,
    ) -> Result<(), JsValue>;
}

pub trait PluginRenderer {
    fn render(&self, _context: &CanvasRenderingContext2d) {}
}

pub trait PluginMouseEvents<T>
where
    T: Renderer + 'static,
{
    /// Is used to implement behaviour of the state if the user clicked inside the specified
    /// html element by the statemachine.
    ///
    /// * `x` - x coordinate of the cursor where the click occured
    /// * `y` - x coordinate of the cursor where the click occured
    /// * `button` - The number of the pressed button (0=left, 1=middle, 2=right) [See here for more informations](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button)
    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, _button: u32, _data: &mut T) {}

    /// Is used to implement behaviour of the state if the user moved the cursor inside the
    /// specified html element by the statemaschine.
    ///
    /// * `x` - x coordinate of the cursor where the click occured
    /// * `y` - x coordinate of the cursor where the click occured
    fn mouse_move(
        &mut self,
        _mouse_pos: Coordinate<f64>,
        _mouse_movement: Coordinate<f64>,
        _data: &mut T,
    ) {
    }
}

pub trait PluginKeyboardEvents {}



pub struct World {
    entities_count: usize,
    plugin_vecs: Vec<Box<dyn PluginVec>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities_count: 0,
            plugin_vecs: Vec::new(),
        }
    }

    pub fn new_entity(&mut self) -> usize {
        let entity_id = self.entities_count;
        for plugin_vec in self.plugin_vecs.iter_mut() {
            plugin_vec.push_none();
        }
        self.entities_count += 1;
        entity_id
    }

    pub fn add_component_to_entity<ComponentType: 'static>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) {
        for plugin_vec in self.plugin_vecs.iter_mut() {
            if let Some(plugin_vec) = plugin_vec
                .as_any_mut()
                .downcast_mut::<Vec<Option<ComponentType>>>()
            {
                plugin_vec[entity] = Some(component);
                return;
            }
        }

        // No matching component storage exists yet, so we have to make one.
        let mut new_plugin_vec: Vec<Option<ComponentType>> =
            Vec::with_capacity(self.entities_count);

        // All existing entities don't have this component, so we give them `None`
        for _ in 0..self.entities_count {
            new_plugin_vec.push(None);
        }

        // Give this Entity the Component.
        new_plugin_vec[entity] = Some(component);
        self.plugin_vecs.push(Box::new(new_plugin_vec));
    }
}

trait PluginVec {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn push_none(&mut self);
}

impl<T: 'static> PluginVec for Vec<Option<T>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn push_none(&mut self) {
        self.push(None)
    }
}
