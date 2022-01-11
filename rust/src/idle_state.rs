use geo::Coordinate;

use crate::{state::{State}, Map};

pub struct IdleState {}

impl Default for IdleState {
    fn default() -> Self {
        IdleState {}
    }
}


impl State for IdleState {


    fn mouse_down(&mut self, _mouse_pos: Coordinate<f64>, _: u32, _: &mut Map) {}

    fn mouse_move(&mut self, _mouse_pos: Coordinate<f64>, _: &mut Map) {}

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, _: u32, _: &mut Map) {}

    fn enter(&self, _: &mut Map) {}

    fn exit(&self, _: &mut Map) {}
}