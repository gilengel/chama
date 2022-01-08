

use idle_state::IdleState;
use map::Map;
use state::State;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

mod intersection;
mod street;
mod district;
mod map;

mod state;
mod idle_state;
mod create_street_state;
mod delete_street_state;
mod create_district_state;

use crate::create_street_state::CreateStreetState;
use crate::delete_street_state::DeleteStreetState;
use crate::create_district_state::CreateDistrictState;

extern crate alloc;


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
    fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue>;
}

#[wasm_bindgen]
pub struct Editor {
    context: CanvasRenderingContext2d,

    render_intersections: bool,
    render_streets: bool,
    state: Box<dyn State>,
    map: Map
}

fn get_canvas_and_context(id: &String) -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), JsValue> {
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
            render_intersections: true,
            render_streets: true,
            state: Box::new(IdleState::default()),
            map: Map::new(width, height)
        }
    }

    
    
    pub fn switch_to_mode(&mut self, mode: u32) {

        match mode {
            0 => log!("idle command, nothing to do"),
            1 => self.state = Box::new(CreateStreetState::new()),
            2 => self.state = Box::new(DeleteStreetState::new()),
            3 => self.state = Box::new(CreateDistrictState::new()),
            _ => log!("unknown command, nothing to do")
        }   
            
    }    

    pub fn width(&self) -> u32 {
        self.map.width()
    }

    pub fn height(&self) -> u32 {
        self.map.height()
    }
    

    pub fn intersections_length(&self) -> usize {
        self.map.intersections_length()
    }

    pub fn streets_length(&self) -> usize {
        self.map.streets_length()
    }

    pub fn set_render_intersections(&mut self, render: bool) {
        self.render_intersections = render;
    }

    pub fn set_render_streets(&mut self, render: bool) {
        self.render_streets = render;
    }

    pub fn render(&self) -> Result<(), JsValue> {       
        self.state.render(&self.map, &self.context)
    }

    pub fn mouse_down(&mut self, x: u32, y: u32, button: u32) {
        self.state.mouse_down(x, y, button, &mut self.map);
    }

    pub fn mouse_up(&mut self, x: u32, y: u32, button: u32) {
        self.state.mouse_up(x, y, button, &mut self.map);
    }

    pub fn mouse_move(&mut self, x: u32, y: u32) {
        self.state.mouse_move(x, y, &mut self.map);
    }
    
}
