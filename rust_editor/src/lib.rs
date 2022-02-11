#![allow(warnings)]


use plugins::{plugin::{Plugin, PluginWithOptions}, camera::Renderer};

pub mod actions;
pub mod gizmo;
pub mod interactive_element;
pub mod macros;
pub mod plugins;
pub mod renderer;
pub mod store;
pub mod style;
pub mod system;
pub mod ui;

#[derive(PartialEq)]
pub enum InformationLayer {
    Debug,
}

pub fn get_plugin<T, S>(plugins: &Vec<Box<dyn PluginWithOptions<T>>>) -> Option<&S>
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

pub fn get_plugin_mut<T, S>(plugins: &mut Vec<Box<dyn PluginWithOptions<T>>>) -> Option<&mut S>
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