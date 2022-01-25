use std::panic;


use geo::Coordinate;
use js_sys::encode_uri_component;


use map::map::InformationLayer;
use map::map::Map;
use state::State;
use states::create_district_state::CreateDistrictState;
use states::create_freeform_street_state::CreateFreeFormStreetState;
use states::create_street_state::CreateStreetState;
use states::delete_district_state::DeleteDistrictState;
use states::delete_street_state::DeleteStreetState;
use states::idle_state::IdleState;
use states::move_control_state::MoveControlState;
use store::Store;

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

mod states;
mod interactive_element;

mod renderer;
mod state;
mod store;
mod style;
mod gizmo;
mod grid;

mod map;

use crate::grid::Grid;


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

pub trait Renderer {
    fn render(
        &self,
        context: &CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
        camera: &Camera,
    ) -> Result<(), JsValue>;
}

#[wasm_bindgen]
pub struct Editor {
    context: CanvasRenderingContext2d,

    additional_information_layers: Vec<InformationLayer>,

    render_intersections: bool,
    render_streets: bool,

    state: Box<dyn State>,
    map: Map,
    grid: Grid,
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

pub struct Camera {
    pub x: i32,
    pub y: i32,

    active: bool,
}

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            x: 0,
            y: 0,
            active: false,
        }
    }
}

#[wasm_bindgen]
impl Editor {
    pub fn new(id: String, width: u32, height: u32) -> Editor {
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        let (_, context) = get_canvas_and_context(&id).unwrap();
        Editor {
            context,
            additional_information_layers: vec![],

            render_intersections: true,
            render_streets: true,
            state: Box::new(IdleState::default()),
            map: Map::new(width, height),
            grid: Grid::default(),
            store: Store::new("fantasy_city_map"),
            camera: Camera::default(),
        }
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
        self.state.exit(&mut self.map);
        match mode {
            0 => log!("idle command, nothing to do"),
            1 => self.state = Box::new(CreateStreetState::new()),
            2 => self.state = Box::new(CreateFreeFormStreetState::new()),
            3 => self.state = Box::new(DeleteStreetState::new()),
            4 => self.state = Box::new(CreateDistrictState::new()),
            5 => self.state = Box::new(CreateFreeFormStreetState::new()),
            6 => self.state = Box::new(DeleteDistrictState::new()),
            7 => self.state = Box::new(MoveControlState::new()),
            
            _ => log!("unknown command, nothing to do"),
        }
        self.state.enter(&mut self.map);
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

        self.state.render(
            &self.map,
            &self.context,
            &self.additional_information_layers,
            &self.camera,
        )
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

        self.state.mouse_down(
            self.transform_cursor_pos_to_grid(x, y, &self.camera),
            button,
            &mut self.map,
        );
    }

    pub fn mouse_up(&mut self, x: u32, y: u32, button: u32) {
        // Camera control via the middle mouse button
        if button == 1 {
            self.camera.active = false;

            return;
        }

        self.state.mouse_up(
            self.transform_cursor_pos_to_grid(x, y, &self.camera),
            button,
            &mut self.map,
        );
    }

    pub fn mouse_move(&mut self, x: u32, y: u32, dx: i32, dy: i32) {
        if self.camera.active {
            self.camera.x += dx;
            self.camera.y += dy;

            return;
        }

        self.state.mouse_move(
            self.transform_cursor_pos_to_grid(x, y, &self.camera),
            &mut self.map,
        );
    }
}
