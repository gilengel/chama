use wasm_bindgen::prelude::wasm_bindgen;

mod systems;
mod actions;
mod state;
mod map;
mod editor;


extern crate alloc;

#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    }
}

#[macro_export]
macro_rules! err {
    ( $( $t:tt )* ) => {
        web_sys::console::error_1(&format!( $( $t )* ).into())
    }
}

#[wasm_bindgen(start)]
pub fn main() {}
