use std::collections::HashMap;

use map::street::Street;
use map::intersection::Intersection;
use uuid::Uuid;
use wasm_bindgen::prelude::wasm_bindgen;

mod systems;
mod actions;
mod state;
mod map;
mod editor;

#[macro_use]
extern crate rust_macro;

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

#[data_source(Street, Intersection)]   
struct Data {}


/*
impl Data {
    pub fn streets(&self) -> &HashMap<Uuid, Street> {
        &self.streets
    }
}
*/

#[wasm_bindgen(start)]
pub fn main() {
    let data = Data::new();
    data.streets();
    data.intersections();
    //let a = data.street(&Uuid::default());

}
