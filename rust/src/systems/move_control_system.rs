use std::ops::Add;

use geo::{Coordinate, Rect};
use uuid::Uuid;

use crate::{
    gizmo::{mouse_over, GetPosition, Gizmo, MoveGizmo, SetPosition},
    interactive_element::{InteractiveElement, InteractiveElementSystem},
    map::map::{InformationLayer, Map},
    state::System,
    Camera, Renderer, renderer::PrimitiveRenderer, style::Style, log, actions::action::Action,
};


pub struct MoveControlSystem {
    hovered_control: Option<Uuid>,
    gizmo: MoveGizmo,
}

impl MoveControlSystem {
    pub fn new() -> Self {
        MoveControlSystem {
            hovered_control: None,

            gizmo: MoveGizmo::new(),
        }
    }

    fn clean_hovered_control_state(&self, map: &mut Map) {
        if let Some(hovered_control) = self.hovered_control {
            map.intersection_mut(&hovered_control)
                .unwrap()
                .set_state(InteractiveElementSystem::Normal);
        }
    }

    fn center_gizmo(&mut self, map: &Map) {
        let elements = map.intersections_with_state(InteractiveElementSystem::Selected);

        let mut sum = Coordinate { x: 0., y: 0. };
        let mut num_elements = 0;
        for x in elements {
            sum = sum.add(x.position());

            num_elements += 1;
        }

        let origin = Coordinate {
            x: sum.x / num_elements as f64,
            y: sum.y / num_elements as f64,
        };

        self.gizmo.set_position(origin);
    }
}



impl System for MoveControlSystem {
    fn mouse_down(&mut self, mouse_pos: Coordinate<f64>, button: u32, map: &mut Map, actions: &mut Vec<Box<dyn Action>>) {
        self.gizmo.mouse_down(
            mouse_pos,
            button,
            map.intersections_with_state_mut(InteractiveElementSystem::Selected),
        );

        if mouse_over(mouse_pos, self.gizmo.position()) {
            return;
        }

        // Single select of intersection
        if map.get_intersection_at_position(&mouse_pos, 50., &vec![]) == None {
            for intersection in map.intersections_with_state_mut(InteractiveElementSystem::Selected)
            {
                intersection.set_state(InteractiveElementSystem::Normal);
            }
        }
    }

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map, actions: &mut Vec<Box<dyn Action>>) {
        self.center_gizmo(map);

        self.gizmo.mouse_move(
            mouse_pos,
            map.intersections_with_state_mut(InteractiveElementSystem::Selected),
        );

        let keys: Vec<Uuid> = map.intersections_keys().map(|x| *x).collect();
        for k in keys {
            map.update_intersection(&k);
        }
    }

    fn mouse_up(&mut self, mouse_pos: Coordinate<f64>, button: u32, map: &mut Map, actions: &mut Vec<Box<dyn Action>>) {
        if self.gizmo.is_active() {
            self.gizmo.mouse_up(
                mouse_pos,
                button,
                map.intersections_with_state_mut(InteractiveElementSystem::Selected),
            );

            return;
        }
    }

    fn blocks_next_systems(&self) -> bool {
        self.gizmo.is_active()
    }

    fn enter(&mut self, map: &mut Map) {
        self.center_gizmo(map);
    }

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
        self.gizmo.render(
            context,
            camera,
            map.intersections_with_state(InteractiveElementSystem::Selected),
        )?;

        Ok(())
    }
}
