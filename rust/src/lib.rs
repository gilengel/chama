use geo::Coordinate;
use map::map_city::City;
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


/*
impl Data {
    pub fn streets(&self) -> &HashMap<Uuid, Street> {
        &self.streets
    }
}
*/

#[wasm_bindgen(start)]
pub fn main() {
    
    let a = City::new();

    let ids : Vec<Uuid> = vec![];
    a.streets_by_ids(&ids);
    a.intersections_at_position(&Coordinate { x: 0., y: 0. }, 40.0);
}
