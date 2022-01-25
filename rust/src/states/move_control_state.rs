use geo::Coordinate;
use uuid::Uuid;

use crate::{
    gizmo::{GetPosition, Gizmo, MoveGizmo, SetPosition},
    interactive_element::{InteractiveElement, InteractiveElementState},
    state::State,
    Camera, Renderer, map::{intersection::Intersection, map::{InformationLayer, Map}},
};

pub struct MoveControlState {
    hovered_control: Option<Uuid>,
    gizmo: MoveGizmo<Intersection>,

    edit_active: bool,
}


impl MoveControlState {
    pub fn new() -> Self {
        MoveControlState {
            hovered_control: None,

            gizmo: MoveGizmo::new(
                |id, map| map.intersection(&id).unwrap(),
                |id, map| map.intersection_mut(&id).unwrap(),
            ),
            edit_active: false,
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
        if let Some(selected) = self.gizmo.element_id {
            let origin = map.intersection(&selected).unwrap().position();

            return self.gizmo.mouse_over(mouse_pos, origin);
        }

        return false;
    }
}

impl State for MoveControlState {
    fn mouse_down(&mut self, mouse_pos: Coordinate<f64>, button: u32, map: &mut Map) {
        if let Some(hovered_control) = self.hovered_control {
            self.gizmo.element_id = Some(hovered_control);
            self.edit_active = true;
        }

        self.gizmo.mouse_down(mouse_pos, button, map);
    }

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map) {
        self.gizmo.mouse_move(mouse_pos, map);

        self.clean_hovered_control_state(map);

        if let Some(hovered_control) = map.get_intersection_at_position(&mouse_pos, 5.0, &vec![]) {
            self.hovered_control = Some(hovered_control);

            if let Some(intersection) = map.intersection_mut(&hovered_control) {
                intersection.set_state(InteractiveElementState::Hover);
            }
        }
    }

    fn mouse_up(&mut self, mouse_pos: Coordinate<f64>, button: u32, map: &mut Map) {
        self.gizmo.mouse_up(mouse_pos, button, map);

        /*
        if map.get_intersection_at_position(&mouse_pos, 5.0, &vec![]) == None {
            self.hovered_control = None;
            self.gizmo.element_id = None;
        }
        */
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

        self.gizmo.render(context, camera, map)?;

        Ok(())
    }
}
