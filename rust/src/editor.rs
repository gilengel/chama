extern crate rust_editor;

use std::panic;



use geo::Coordinate;
use js_sys::encode_uri_component;
use rust_editor::{grid::Grid, store::Store, camera::Camera, InformationLayer};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{
    actions::action::Action,
    err,
    log,
    map::map::Map,
    state::System,
    systems::{
        box_select_system::BoxSelectSystem, create_district_system::CreateDistrictSystem,
        create_freeform_street_system::CreateFreeFormStreetSystem,
        create_street_system::CreateStreetSystem, delete_district_system::DeleteDistrictSystem,
        delete_street_system::DeleteStreetSystem, move_control_system::MoveControlSystem,
        render_map_system::MapRenderSystem,
    },
};

#[wasm_bindgen]
pub struct Editor {
    context: CanvasRenderingContext2d,

    additional_information_layers: Vec<InformationLayer>,

    render_intersections: bool,
    render_streets: bool,

    active_systems: Vec<Box<dyn System + Send + Sync>>,
    undo_stack: Vec<Box<dyn Action>>,
    redo_stack: Vec<Box<dyn Action>>,
    map: Map,
    grid: rust_editor::grid::Grid,
    camera: Camera,
    store: Option<Store>,
}

fn get_canvas_and_context(
    id: &String,
) -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    Ok((canvas, context))
}

#[wasm_bindgen]
impl Editor {
    pub fn new(id: String, width: u32, height: u32) -> Editor {
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        let (_, context) = get_canvas_and_context(&id).unwrap();
        Editor {
            context,
            additional_information_layers: vec![],

            active_systems: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),

            render_intersections: true,
            render_streets: true,
            map: Map::new(width, height),
            grid: Grid::default(),
            store: Store::new("fantasy_city_map"),
            camera: Camera::default(),
        }
    }

    pub fn is_undoable(&self) -> bool {
        log!("{:?}", self.undo_stack.is_empty());
        !self.undo_stack.is_empty()
    }

    pub fn is_redoable(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) {
        if let Some(action) = self.undo_stack.last_mut() {
            (**action).undo(&mut self.map);
        }

        self.redo_stack.push(self.undo_stack.pop().unwrap());
    }

    pub fn redo(&mut self) {
        if let Some(action) = self.redo_stack.last_mut() {
            (**action).redo(&mut self.map);
        }

        self.undo_stack.push(self.redo_stack.pop().unwrap());
    }

    pub fn deactivate_system(&mut self) {
        todo!();
    }

    pub fn deactivate_all_systems(&mut self) {
        self.active_systems.clear();
    }

    pub fn import(&mut self, content: String) {
        match serde_json::from_str::<Map>(&content) {
            Ok(m) => self.map = m,
            Err(e) => err!("{:?}", e),
        }
    }

    /// Generates a file locally containing the current map as a JSON file and triggers the browser file download dialog
    pub fn download(&self) -> Result<(), JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();

        let element = document
            .create_element("a")
            .unwrap()
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .unwrap();

        let mut link = "data:text/plain;charset=utf-8,".to_owned();
        let content: String =
            encode_uri_component(&serde_json::to_string(&self.map).unwrap()).into();
        link.push_str(&content);
        //content.push_string();
        element.set_attribute("href", &link)?;
        element.set_attribute("download", "yourfantasymap.json")?;

        document.body().unwrap().append_child(&element)?;
        element.click();

        document.body().unwrap().remove_child(&element)?;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn save(&self) -> Result<(), JsValue> {
        self.store.as_ref().unwrap().sync_local_storage(&self.map)
    }

    pub fn load(&mut self) {
        self.map = self.store.as_ref().unwrap().fetch_local_storage().unwrap();
    }

    pub fn set_enable_debug_information(&mut self, enable_debug_information: bool) {
        if !enable_debug_information {
            if let Some(index) = self
                .additional_information_layers
                .iter()
                .position(|x| *x == InformationLayer::Debug)
            {
                self.additional_information_layers.remove(index);
            }
            return;
        }

        if self
            .additional_information_layers
            .iter()
            .position(|x| *x == InformationLayer::Debug)
            == None
        {
            self.additional_information_layers
                .push(InformationLayer::Debug);
        }
    }

    pub fn switch_to_mode(&mut self, mode: u32) {
        self.deactivate_all_systems();

        let mut new_systems: Vec<Box<dyn System + Send + Sync>> = Vec::new();
        new_systems.push(Box::new(MapRenderSystem::new()));

        match mode {
            1 => new_systems.push(Box::new(CreateStreetSystem::new())),
            3 => new_systems.push(Box::new(DeleteStreetSystem::new())),
            4 => new_systems.push(Box::new(CreateDistrictSystem::new())),
            2 => new_systems.push(Box::new(CreateFreeFormStreetSystem::new())),
            5 => new_systems.push(Box::new(CreateFreeFormStreetSystem::new())),
            6 => new_systems.push(Box::new(DeleteDistrictSystem::new())),
            7 => {
                new_systems.push(Box::new(MoveControlSystem::new()));
                new_systems.push(Box::new(BoxSelectSystem::new()));
            }
            8 => new_systems.push(Box::new(BoxSelectSystem::new())),
            _ => log!("unknown command, nothing to do"),
        };

        self.active_systems = new_systems;
    }

    pub fn set_grid_enabled(&mut self, enabled: bool) {
        self.grid.set_enabled(enabled);
    }

    pub fn set_grid_offset(&mut self, offset: f64) {
        self.grid.set_offset(offset as u32);
    }

    pub fn set_grid_subdivisions(&mut self, subdivisions: f64) {
        let mut subdivisions = subdivisions as u8;

        if subdivisions == 0 {
            subdivisions = 1;
        }

        self.grid.set_subdivisions(subdivisions);
    }

    pub fn width(&self) -> u32 {
        self.map.width()
    }

    pub fn height(&self) -> u32 {
        self.map.height()
    }

    pub fn intersections_length(&self) -> usize {
        self.map.intersections().len()
    }

    pub fn streets_length(&self) -> usize {
        self.map.streets().len()
    }

    pub fn set_render_intersections(&mut self, render: bool) {
        self.render_intersections = render;
    }

    pub fn set_render_streets(&mut self, render: bool) {
        self.render_streets = render;
    }

    pub fn render(&self) -> Result<(), JsValue> {
        self.context
            .clear_rect(0.0, 0.0, self.map.width().into(), self.map.height().into());

        if self.grid.is_enabled() {
            let offset = self.grid.offset() as i32;
            let x: i32 = self.camera.x % offset - offset;
            let y: i32 = self.camera.y % offset - offset;

            self.context.translate(x as f64, y as f64)?;

            self.grid.render(
                &self.context,
                self.map.width() + (offset as f64 * 2.) as u32,
                self.map.height() + (offset as f64 * 2.) as u32,
            )?;

            self.context.set_transform(1., 0., 0., 1., 0., 0.)?;
        }

        for system in &self.active_systems {
            system.render(
                &self.map,
                &self.context,
                &self.additional_information_layers,
                &self.camera,
            )?;

            if system.blocks_next_systems() {
                break;
            }
        }

        Ok(())
    }

    fn transform_cursor_pos_to_grid(&self, x: u32, y: u32, camera: &Camera) -> Coordinate<f64> {
        if !self.grid.is_enabled() {
            return Coordinate {
                x: (x as i32 - camera.x).into(),
                y: (y as i32 - camera.y).into(),
            };
        }

        let factor = self.grid.offset() as f32 / self.grid.subdivisions() as f32;
        let x = ((x as i32 - camera.x) as f32 / factor).round();
        let y = ((y as i32 - camera.y) as f32 / factor).round();

        Coordinate {
            x: (x * factor as f32).into(),
            y: (y * factor as f32).into(),
        }
    }

    pub fn mouse_down(&mut self, x: u32, y: u32, button: u32) {
        // Camera control via the middle mouse button
        if button == 1 {
            self.camera.active = true;

            return;
        }

        let mouse_pos = self.transform_cursor_pos_to_grid(x, y, &self.camera);
        for system in &mut self.active_systems {
            system.mouse_down(mouse_pos, button, &mut self.map, &mut self.undo_stack);

            if system.blocks_next_systems() {
                break;
            }
        }
    }

    pub fn mouse_up(&mut self, x: u32, y: u32, button: u32) {
        // Camera control via the middle mouse button
        if button == 1 {
            self.camera.active = false;

            return;
        }

        let mouse_pos = self.transform_cursor_pos_to_grid(x, y, &self.camera);
        for system in &mut self.active_systems {
            system.mouse_up(mouse_pos, button, &mut self.map, &mut self.undo_stack);

            if system.blocks_next_systems() {
                break;
            }
        }
    }

    pub fn mouse_move(&mut self, x: u32, y: u32, dx: i32, dy: i32) {
        if self.camera.active {
            self.camera.x += dx;
            self.camera.y += dy;

            return;
        }

        let mouse_pos = self.transform_cursor_pos_to_grid(x, y, &self.camera);

        for system in &mut self.active_systems {
            system.mouse_move(mouse_pos, &mut self.map, &mut self.undo_stack);

            if system.blocks_next_systems() {
                break;
            }
        }
    }
}
