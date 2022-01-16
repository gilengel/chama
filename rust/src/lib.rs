use geo::Coordinate;
use idle_state::IdleState;
use js_sys::encode_uri_component;
use js_sys::JsString;
use map::InformationLayer;
use map::Map;
use state::State;
use store::Store;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

mod create_district_state;
mod create_street_state;
mod delete_district_state;
mod delete_street_state;
mod district;
mod house;
mod idle_state;
mod interactive_element;
mod intersection;
mod map;
mod renderer;
mod state;
mod store;
mod street;
mod style;

use crate::create_district_state::CreateDistrictState;
use crate::create_street_state::CreateStreetState;
use crate::delete_district_state::DeleteDistrictState;
use crate::delete_street_state::DeleteStreetState;

extern crate alloc;

#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

//#[cfg(feature = "wee_alloc")]
//#[global_allocator]
//static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    Ok(())
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    }
}

pub trait Renderer {
    fn render(
        &self,
        context: &CanvasRenderingContext2d,
        additional_information_layer: &Vec<InformationLayer>,
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
    store: Option<Store>,
}

struct Grid {
    offset: u32,
    subdivisions: u8,
    enabled: bool,
}

impl Grid {
    pub fn render(
        &self,
        context: &CanvasRenderingContext2d,
        width: u32,
        height: u32,
    ) -> Result<(), JsValue> {
        if self.offset == 0 {
            return Ok(());
        }

        context.save();
        context.set_line_width(2.0);
        context.set_stroke_style(&"rgb(40, 40, 40)".into());

        let steps_x = (width as f64 / self.offset as f64).ceil() as u32;
        let steps_y = (height as f64 / self.offset as f64).ceil() as u32;

        let sub_offset = (self.offset as f64 / self.subdivisions as f64).ceil() as u32;

        for i in 0..steps_x {
            let i = (i * self.offset).into();

            context.save();
            context.set_line_width(1.0);
            for k in 0..self.subdivisions as u32 {
                context.begin_path();
                context.move_to(i + (k * sub_offset) as f64, 0.0);
                context.line_to(i + (k * sub_offset) as f64, height.into());
                context.close_path();
                context.stroke();
            }
            context.restore();

            context.set_line_width(4.0);
            context.begin_path();
            context.move_to(i, 0.0);
            context.line_to(i, height.into());
            context.close_path();
            context.stroke();
        }

        for i in 0..steps_y {
            let i = (i * self.offset).into();

            context.save();
            context.set_line_width(1.0);
            for k in 0..self.subdivisions as u32 {
                context.begin_path();
                context.move_to(0., i + (k * sub_offset) as f64);
                context.line_to(width.into(), i + (k * sub_offset) as f64);
                context.close_path();
                context.stroke();
            }
            context.restore();

            context.set_line_width(2.0);
            context.begin_path();
            context.move_to(0.0, i);
            context.line_to(width.into(), i);
            context.close_path();
            context.stroke();
        }

        context.restore();

        Ok(())
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            offset: 200,
            subdivisions: 4,
            enabled: true,
        }
    }
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

    pub fn save(&self) {
        self.store.as_ref().unwrap().sync_local_storage(&self.map);
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
            2 => self.state = Box::new(DeleteStreetState::new()),
            3 => self.state = Box::new(CreateDistrictState::new()),
            4 => self.state = Box::new(DeleteDistrictState::new()),
            _ => log!("unknown command, nothing to do"),
        }
        self.state.enter(&mut self.map);
    }

    pub fn set_grid_enabled(&mut self, enabled: bool) {
        self.grid.enabled = enabled
    }

    pub fn set_grid_offset(&mut self, offset: f64) {
        self.grid.offset = offset as u32;
    }

    pub fn set_grid_subdivisions(&mut self, subdivisions: f64) {
        let mut subdivisions = subdivisions as u8;

        if subdivisions == 0 {
            subdivisions = 1;
        }

        self.grid.subdivisions = subdivisions;
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

        if self.grid.enabled {
            self.grid
                .render(&self.context, self.map.width(), self.map.height())?
        }

        self.state.render(
            &self.map,
            &self.context,
            &self.additional_information_layers,
        )
    }

    fn transform_cursor_pos_to_grid(&self, x: u32, y: u32) -> Coordinate<f64> {
        if !self.grid.enabled {
            return Coordinate {
                x: x.into(),
                y: y.into(),
            };
        }

        let factor = self.grid.offset as f32 / self.grid.subdivisions as f32;
        let x = (x as f32 / factor).round();
        let y = (y as f32 / factor).round();

        Coordinate {
            x: (x * factor).into(),
            y: (y * factor).into(),
        }
    }

    pub fn mouse_down(&mut self, x: u32, y: u32, button: u32) {
        self.state.mouse_down(
            self.transform_cursor_pos_to_grid(x, y),
            button,
            &mut self.map,
        );
    }

    pub fn mouse_up(&mut self, x: u32, y: u32, button: u32) {
        self.state.mouse_up(
            self.transform_cursor_pos_to_grid(x, y),
            button,
            &mut self.map,
        );
    }

    pub fn mouse_move(&mut self, x: u32, y: u32) {
        self.state
            .mouse_move(self.transform_cursor_pos_to_grid(x, y), &mut self.map);
    }
}
