use std::{collections::HashMap, ops::Add};

use geo::Coordinate;
use rust_editor::{
    gizmo::{mouse_over, GetPosition, Gizmo, MoveGizmo, SetPosition},
    interactive_element::{InteractiveElement, InteractiveElementState},
    plugins::plugin::PluginWithOptions,
    system::System,
    InformationLayer,
};
use uuid::Uuid;

use crate::{map::map::Map, Modes};

pub struct MoveControlSystem {
    hovered_control: Option<Uuid>,
    gizmo: MoveGizmo,
}

impl MoveControlSystem {
    fn clean_hovered_control_state(&self, map: &mut Map) {
        if let Some(hovered_control) = self.hovered_control {
            map.intersection_mut(&hovered_control)
                .unwrap()
                .set_state(InteractiveElementState::Normal);
        }
    }

    fn center_gizmo(&mut self, map: &Map) {
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
}

impl System<Map, Modes> for MoveControlSystem {
    fn mouse_down(
        &mut self,
        mouse_pos: Coordinate<f64>,
        button: u32,
        map: &mut Map,

        _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) {
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
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        map: &mut Map,

        _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) {
        self.center_gizmo(map);

        self.gizmo.mouse_move(
            mouse_pos,
            map.intersections_with_state_mut(InteractiveElementState::Selected),
        );

        let keys: Vec<Uuid> = map.intersections_keys().map(|x| *x).collect();
        for k in keys {
            map.update_intersection(&k);
        }
    }

    fn mouse_up(
        &mut self,
        mouse_pos: Coordinate<f64>,
        button: u32,
        map: &mut Map,

        _plugins: &mut HashMap<&'static str, Box<dyn PluginWithOptions<Map, Modes>>>,
    ) {
        if self.gizmo.is_active() {
            self.gizmo.mouse_up(
                mouse_pos,
                button,
                map.intersections_with_state_mut(InteractiveElementState::Selected),
            );

            return;
        }
    }

    fn blocks_next_systems(&self) -> bool {
        self.gizmo.is_active()
    }

    fn enter(
        &mut self,
        data: &mut Map,
        _plugins: HashMap<&'static str, &mut Box<(dyn PluginWithOptions<Map, Modes> + 'static)>>,
    ) {
        self.center_gizmo(data);
    }

    fn exit(
        &self,
        data: &mut Map,
        _plugins: HashMap<&'static str, &mut Box<(dyn PluginWithOptions<Map, Modes> + 'static)>>,
    ) {
        self.clean_hovered_control_state(data);
    }

    fn render(
        &self,
        map: &Map,
        context: &web_sys::CanvasRenderingContext2d,
        _additional_information_layer: &Vec<InformationLayer>,
        _plugins: &HashMap<&'static str, Box<(dyn PluginWithOptions<Map, Modes> + 'static)>>,
    ) -> Result<(), wasm_bindgen::JsValue> {
        self.gizmo.render(
            context,
            map.intersections_with_state(InteractiveElementState::Selected),
        )?;

        Ok(())
    }
}
