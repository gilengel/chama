use crate::{state::{State}, Map};

pub struct IdleState {}

impl Default for IdleState {
    fn default() -> Self {
        IdleState {}
    }
}


impl State for IdleState {


    fn mouse_down(&mut self, _: u32, _: u32, _: u32, _: &mut Map) {}

    fn mouse_move(&mut self, _: u32, _: u32, _: &mut Map) {}

    fn mouse_up(&mut self, _: u32, _: u32, _: u32, _: &mut Map) {}

    fn update(&mut self) {}

    fn enter(&self) {}

    fn exit(&self) {}
}