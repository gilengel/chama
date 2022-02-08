use std::{cell::RefCell, rc::Rc};

use editor::{Editor, window, document};
use plugins::camera::Renderer;
use wasm_bindgen::{prelude::Closure, JsValue, JsCast};

pub mod actions;
pub mod editor;
pub mod gizmo;
pub mod grid;
pub mod interactive_element;
pub mod macros;
pub mod plugins;
pub mod renderer;
pub mod store;
pub mod style;
pub mod system;
pub mod toolbar;
pub mod ui;

#[derive(PartialEq)]
pub enum InformationLayer {
    Debug,
}


pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}


pub fn launch<T>(editor: Rc<RefCell<Editor<T>>>) -> Result<(), JsValue>
where
    T: Renderer + Default + 'static,
{
    let document = document();

    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    document.body().unwrap().append_child(&canvas)?;
    canvas.set_width(1920);
    canvas.set_height(1080);
    {
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

        editor.borrow_mut().context = Some(context);
    }

    {
        let editor = editor.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            editor.borrow_mut().mouse_down(
                event.client_x() as u32,
                event.client_y() as u32,
                event.button() as u32,
            );
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let editor = editor.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            editor.borrow_mut().mouse_up(
                event.client_x() as u32,
                event.client_y() as u32,
                event.button() as u32,
            );
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let editor = editor.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            // TODO reenable the movement difference
            editor.borrow_mut().mouse_move(
                event.client_x() as u32,
                event.client_y() as u32,
                event.movement_x() as i32,
                event.movement_y() as i32,
            );
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            editor.borrow().render().expect("Error while rendering");
            // Schedule ourself for another requestAnimationFrame callback.
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }

    // TODO forget leaks memory we need a more intelligent way to handle this

    let main = document
        .get_element_by_id("main")
        .expect("expecting an element with id \"main\"");
    main.append_child(&canvas)?;

    Ok(())
}