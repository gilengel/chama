pub mod actions;
pub mod camera;
pub mod editor;
pub mod gizmo;
pub mod grid;
pub mod interactive_element;
pub mod renderer;
pub mod store;
pub mod style;
pub mod system;
pub mod macros;

#[derive(PartialEq)]
pub enum InformationLayer {
    Debug,
}