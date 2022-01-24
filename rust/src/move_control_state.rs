use geo::Coordinate;
use uuid::Uuid;

use crate::{
    gizmo::{Gizmo, MoveGizmo, GetPosition, SetPosition},
    interactive_element::{InteractiveElement, InteractiveElementState},
    map::{InformationLayer, Map},
    state::State,
    Camera, Renderer,
};

pub struct MoveControlState {
    hovered_control: Option<Uuid>,
    selected_control: Option<Uuid>,
    gizmo: MoveGizmo,

    edit_active: bool
}

impl MoveControlState {
    pub fn new() -> Self {
        MoveControlState {
            hovered_control: None,
            selected_control: None,

            gizmo: MoveGizmo::new(),
            edit_active: false
        }
    }

    fn clean_hovered_control_state(&self, map: &mut Map) {
        if let Some(hovered_control) = self.hovered_control {
            map.intersection_mut(&hovered_control)
                .unwrap()
                .set_state(InteractiveElementState::Normal);
        }
    }

    fn over_gizmo(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map) -> bool {
        if let Some(selected) = self.selected_control {
            let origin = map.intersection(&selected).unwrap().position();

            return self.gizmo.mouse_over(mouse_pos, origin);
        }

        return false
    }
}

impl State for MoveControlState {
    fn mouse_down(&mut self, mouse_pos: Coordinate<f64>, _: u32, map: &mut Map) {
        if self.over_gizmo(mouse_pos, map) {
            self.edit_active = true;
            return;
        }

        if let Some(hovered_control) = self.hovered_control {
            self.selected_control = Some(hovered_control);
            self.edit_active = true;
        }
    }

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map) {
        if self.edit_active && self.gizmo.affected_axis.is_some() {
            let old_position = map.intersection(&self.selected_control.unwrap()).unwrap().position();
            let new_position = match self.gizmo.affected_axis.as_ref().unwrap() {
                crate::gizmo::Axis::X => Coordinate { x: mouse_pos.x, y: old_position.y },
                crate::gizmo::Axis::Y => Coordinate { x: old_position.x, y: mouse_pos.y },
                crate::gizmo::Axis::XY => mouse_pos,
            };
            map.intersection_mut(&self.selected_control.unwrap()).unwrap().set_position(new_position);

            let a = map.intersections().clone();
            let keys = a.keys();
            for k in keys {
                map.update_intersection(&k);
            }


        }
        

        self.clean_hovered_control_state(map);

        if let Some(hovered_control) = map.get_intersection_at_position(&mouse_pos, 5.0, &vec![]) {
            self.hovered_control = Some(hovered_control);

            if let Some(intersection) = map.intersection_mut(&hovered_control) {
                intersection.set_state(InteractiveElementState::Hover);
            }
        }
    }

    fn mouse_up(&mut self, mouse_pos: Coordinate<f64>, _: u32, map: &mut Map) {
        if self.edit_active {
            self.edit_active = false;
            return; 

        }
        if map.get_intersection_at_position(&mouse_pos, 5.0, &vec![]) == None {
            self.hovered_control = None;
            self.selected_control = None;
        }
    }

    fn enter(&self, _map: &mut Map) {}

    fn exit(&self, map: &mut Map) {
        self.clean_hovered_control_state(map);
    }

    fn render(
        &self,
        map: &Map,
        context: &web_sys::CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
        camera: &Camera,
    ) -> Result<(), wasm_bindgen::JsValue> {
        map.render(context, additional_information_layer, camera)?;

        if let Some(selected) = self.selected_control {
            let position = map.intersection(&selected).unwrap().position();
            context.translate(position.x, position.y)?;

            self.gizmo.render(context, camera)?;

            context.set_transform(1., 0., 0., 1., 0., 0.)?;
        }
        Ok(())
    }
}
