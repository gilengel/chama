#![feature(trait_alias)]

pub mod actions;
pub mod camera;
pub mod editor;
pub mod gizmo;
pub mod grid;
pub mod interactive_element;
pub mod macros;
pub mod plugins;
pub mod renderer;
pub mod store;
pub mod style;
pub mod system;

#[derive(PartialEq)]
pub enum InformationLayer {
    Debug,
}
