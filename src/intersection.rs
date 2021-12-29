use bevy::math::Vec2;

use crate::street::Street;

pub struct Intersection {
    pub position: Vec2,
    //connected_streets: Vec<Street<'a>>
}

impl Intersection {
    pub fn add_incoming_street(&mut self, street: &Street) {

    }

    pub fn remove_incoming_street(&mut self, street: &Street) {

    }

    pub fn add_outgoing_street(&mut self, street: &Street) {

    }

    pub fn remove_outgoing_street(&mut self, street: &Street) {

    }
}