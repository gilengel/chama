use std::ops::Add;

use geo::{Coordinate, Rect};
use uuid::Uuid;

use crate::{
    gizmo::{mouse_over, GetPosition, Gizmo, MoveGizmo, SetPosition},
    interactive_element::{InteractiveElement, InteractiveElementState},
    map::map::{InformationLayer, Map},
    state::State,
    Camera, Renderer, renderer::PrimitiveRenderer, style::Style, log,
};

fn default_rect() -> Rect<f64> {
    Rect::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 0., y: 0. })
}

pub struct MoveControlState {
    hovered_control: Option<Uuid>,
    gizmo: MoveGizmo,
    selection_min: Coordinate<f64>,
    selection_max: Coordinate<f64>,
    //selection_active: bool,
}

impl MoveControlState {
    pub fn new() -> Self {
        MoveControlState {
            hovered_control: None,

            gizmo: MoveGizmo::new(),
            selection_min: Coordinate { x: 0., y: 0. },
            selection_max: Coordinate { x: 0., y: 0. },
        }
    }

    fn clean_hovered_control_state(&self, map: &mut Map) {
        if let Some(hovered_control) = self.hovered_control {
            map.intersection_mut(&hovered_control)
                .unwrap()
                .set_state(InteractiveElementState::Normal);
        }
    }
}



impl State for MoveControlState {
    fn mouse_down(&mut self, mouse_pos: Coordinate<f64>, button: u32, map: &mut Map) {
        self.gizmo.mouse_down(
            mouse_pos,
            button,
            map.intersections_with_state_mut(InteractiveElementState::Selected),
        );

        if mouse_over(mouse_pos, self.gizmo.position()) {
            return;
        }

        // Single select of intersection
        if map.get_intersection_at_position(&mouse_pos, 50., &vec![]) == None {
            for intersection in map.intersections_with_state_mut(InteractiveElementState::Selected)
            {
                intersection.set_state(InteractiveElementState::Normal);
            }
        }

        // At this point we assume the user want to select multiple entities
        self.selection_min = mouse_pos;
    }

    fn mouse_move(&mut self, mouse_pos: Coordinate<f64>, map: &mut Map) {
        self.gizmo.mouse_move(
            mouse_pos,
            map.intersections_with_state_mut(InteractiveElementState::Selected),
        );

        let keys: Vec<Uuid> = map.intersections_keys().map(|x| *x).collect();
        for k in keys {
            map.update_intersection(&k);
        }
        
        if self.selection_min != (Coordinate { x: 0., y: 0. }) {
            self.selection_max = mouse_pos;
        }
    }

    fn mouse_up(&mut self, mouse_pos: Coordinate<f64>, button: u32, map: &mut Map) {
        if self.gizmo.is_active() {
            self.gizmo.mouse_up(
                mouse_pos,
                button,
                map.intersections_with_state_mut(InteractiveElementState::Selected),
            );

            return;
        }

        for intersection in map.intersections_within_rectangle_mut(&Rect::new(self.selection_min, self.selection_max)) {
            intersection.set_state(InteractiveElementState::Selected);
        }

        self.selection_min = Coordinate { x: 0., y: 0.};
        self.selection_max = Coordinate { x: 0., y: 0.};
    }

    fn enter(&mut self, map: &mut Map) {
        let elements = map.intersections_with_state(InteractiveElementState::Selected);

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

        self.gizmo.render(
            context,
            camera,
            map.intersections_with_state(InteractiveElementState::Selected),
        )?;

        Rect::new(self.selection_min, self.selection_max).render(
            &Style {
                border_width: 2,
                border_color: "rgba(255, 255, 255, 0.1)".to_string(),
                background_color: "rgba(255, 255, 255, 0.05)".to_string(),
            },
            context,
        )?;

        Ok(())
    }
}
