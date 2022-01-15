use geo::Coordinate;
use idle_state::IdleState;
use map::InformationLayer;
use map::Map;
use state::State;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

mod district;
mod interactive_element;
mod intersection;
mod map;
mod street;

mod style;

mod create_district_state;
mod create_street_state;
mod delete_district_state;
mod delete_street_state;
mod idle_state;
mod state;
mod renderer;
mod house;

use crate::create_district_state::CreateDistrictState;
use crate::create_street_state::CreateStreetState;
use crate::delete_district_state::DeleteDistrictState;
use crate::delete_street_state::DeleteStreetState;

extern crate alloc;

#[macro_export] macro_rules! log {
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
    fn render(&self, context: &CanvasRenderingContext2d, additional_information_layer: &Vec<InformationLayer>) -> Result<(), JsValue>;
}

#[wasm_bindgen]
pub struct Editor {
    context: CanvasRenderingContext2d,

    additional_information_layers: Vec<InformationLayer>,

    render_intersections: bool,
    render_streets: bool,
    
    state: Box<dyn State>,
    map: Map,
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
        }
    }

    pub fn set_enable_debug_information(&mut self, enable_debug_information: bool) {
        if !enable_debug_information {
            if let Some(index) = self.additional_information_layers.iter().position(|x| *x == InformationLayer::Debug) {
                self.additional_information_layers.remove(index);
            }
            return
        }

        if self.additional_information_layers.iter().position(|x| *x == InformationLayer::Debug) == None {
            self.additional_information_layers.push(InformationLayer::Debug);
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
        self.state.render(&self.map, &self.context, &self.additional_information_layers)
    }

    pub fn mouse_down(&mut self, x: u32, y: u32, button: u32) {
        self.state.mouse_down(
            Coordinate {
                x: x.into(),
                y: y.into(),
            },
            button,
            &mut self.map,
        );
    }

    pub fn mouse_up(&mut self, x: u32, y: u32, button: u32) {
        self.state.mouse_up(
            Coordinate {
                x: x.into(),
                y: y.into(),
            },
            button,
            &mut self.map,
        );
    }

    pub fn mouse_move(&mut self, x: u32, y: u32) {
        self.state.mouse_move(
            Coordinate {
                x: x.into(),
                y: y.into(),
            },
            &mut self.map,
        );
    }
}
