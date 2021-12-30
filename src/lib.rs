use std::cell::Cell;
use std::cell::RefCell;
use std::cell::RefMut;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use geo_types::Point;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

#[derive(Clone, Debug)]
struct Street {
    start: Point<f64>,
    end: Point<f64>,
}

impl Street {
    pub fn render(&self, context: &CanvasRenderingContext2d) {
        context.begin_path();
        context
            .move_to(self.start.x(), self.start.y());
        context
            .line_to(self.end.x(), self.end.y());
        context.stroke();
    }
}

impl Default for Street {
    fn default() -> Self { Street { start: Point::new(0.0, 0.0), end: Point::new(0.0, 0.0) } }
}

#[wasm_bindgen]
pub struct Editor {
    width: u32,
    height: u32,
    streets: Vec<Street>,

    temp_street: Street,

    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,

    mouse_pressed: bool,
}

fn get_canvas_and_context() -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("map_canvas").unwrap();
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
    pub fn new() -> Editor {
        let (canvas, context) = get_canvas_and_context().unwrap();
        Editor {
            width: 1920,
            height: 800,
            streets: vec![],
            temp_street: Street {
                start: Point::new(0.0, 0.0),
                end: Point::new(0.0, 0.0),
            },
            canvas,
            context,
            mouse_pressed: false
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn render(&self) {
        self.context.clear_rect(0.0, 0.0, 1920.0, 1080.0);

        self.temp_street.render(&self.context);

        for street in &self.streets {
            street.render(&self.context);
        }
    }

    pub fn mouse_down(&mut self, x: u32, y: u32) {
        self.temp_street.start.set_x(x as f64);
        self.temp_street.start.set_y(y as f64);

        self.temp_street.end.set_x(x as f64);
        self.temp_street.end.set_y(y as f64);

        self.mouse_pressed = true
    }

    pub fn mouse_up(&mut self, x: u32, y: u32) {
        if self.mouse_pressed {
            self.temp_street.end.set_x(x as f64);
            self.temp_street.end.set_y(y as f64);

            self.streets.push(self.temp_street.clone());

            self.temp_street = Street::default();
        }

        self.mouse_pressed = false;
    }

    pub fn mouse_move(&mut self, x: u32, y: u32) {
        if self.mouse_pressed {
            self.temp_street.end.set_x(x as f64);
            self.temp_street.end.set_y(y as f64);
        }
    }

    /*

    pub fn register_mouse_events(&self) -> Result<(), JsValue>  {
        let (canvas, context) = self.get_canvas_and_context().unwrap();

        //let mut temp_street = Rc::new(Cell::new(Street { start: Point::new(0.0, 0.0), end: Point::new(0.0, 0.0) }));
        let start = Rc::new(Cell::new(Point::new(0.0, 0.0)));
        let end = Rc::new(Cell::new(Point::new(0.0, 0.0)));
        let pressed = Rc::new(Cell::new(false));
        {
            let pressed = pressed.clone();
            let start = start.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {

                start.set(Point::new(event.offset_x() as f64, event.offset_y() as f64));

                pressed.set(true);
            }) as Box<dyn FnMut(_)>);
            canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        {
            let context = context.clone();
            let start = start.clone();
            let end = end.clone();
            let pressed = pressed.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                if pressed.get() {
                    end.set(Point::new(event.offset_x() as f64, event.offset_y() as f64));
                    context.clear_rect(0.0, 0.0, 1920.0, 1080.0);
                    context.begin_path();
                    context.move_to(start.get().x(), start.get().y());
                    context.line_to(end.get().x(), end.get().y());
                    context.stroke();

                }
            }) as Box<dyn FnMut(_)>);
            canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        {
            let context = context.clone();
            let pressed = pressed.clone();
            let start = start.clone();
            let end = end.clone();
            //let streets = Rc<RefCell<_>> = Rc::new(RefCell::new(self.streets));
            let streets: Rc<RefCell<_>> = Rc::new(RefCell::new(self.streets.clone()));

            //let mut streets = Cell::new(self.streets);
            let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                pressed.set(false);
                context.line_to(event.offset_x() as f64, event.offset_y() as f64);
                context.stroke();

                let mut map: RefMut<_> = streets.borrow_mut();
                map.push(Street { start: start.get(), end: end.get() });
                //web_sys::console::log_1(&streets.len().to_string().into());
            }) as Box<dyn FnMut(_)>);
            canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        Ok(())
    }
    */
}
