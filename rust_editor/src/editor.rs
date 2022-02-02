use std::fmt::Debug;
use std::hash::Hash;
use std::{cell::RefCell, collections::HashMap, panic, rc::Rc};

use geo::Coordinate;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{
    CanvasRenderingContext2d, HtmlElement, HtmlInputElement, HtmlLabelElement, HtmlSpanElement,
};

use crate::plugins::Plugin;
use crate::{
    actions::Action,
    camera::{Camera, Renderer},
    html, html_impl,
    macros::slugify,
    system::System,
    InformationLayer,
};

pub trait EditorMode = Copy + Clone;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ToolbarPosition {
    Top,
    Right,
    Bottom,
    Left,
}

pub struct Toolbar {
    pub buttons: Vec<ToolbarButton>,
}

pub struct ToolbarButton {
    pub icon: &'static str,
    pub tooltip: &'static str,
    pub value: u8,
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

pub fn add_mode<T>(
    editor: Rc<RefCell<Editor<T>>>,
    mode: u8,
    systems: Vec<Box<dyn System<T> + Send + Sync>>,
) where
    T: Renderer + Default + 'static,
{
    if editor.borrow().modes.contains_key(&mode) {
        // TODO Error handling
        todo!()
    }

    editor.borrow_mut().modes.insert(mode as u8, systems);
}

pub fn add_toolbar<T>(
    editor: Rc<RefCell<Editor<T>>>,
    toolbar: Toolbar,
    position: ToolbarPosition,
) -> Result<(), JsValue>
where
    T: Renderer + Default + 'static,
{
    let container_id = match position {
        ToolbarPosition::Top => "top_primary_toolbar",
        ToolbarPosition::Right => "right_primary_toolbar",
        ToolbarPosition::Bottom => "bottom_primary_toolbar",
        ToolbarPosition::Left => "left_primary_toolbar",
    };

    {
        let mut editor = editor.borrow_mut();
        if editor.toolbars.get(&position).is_none() {
            editor.toolbars.insert(position.clone(), vec![]);

            html! {
                ["main"]

                <div id=(container_id)></div>
            }
        }
    }

    let document = document();
    let ul = document
        .create_element("ul")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();

    ul.set_class_name("toolbar");
    ul.set_attribute("role", "radiogroup")?;

    let a = Closure::wrap(Box::new(move |evt: web_sys::Event| {
        let value = evt
            .target()
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap()
            .value();

        let value = value.as_bytes()[0];

        editor.borrow_mut().active_mode = value;
    }) as Box<dyn FnMut(_)>);

    for i in &toolbar.buttons {
        let li = document
            .create_element("li")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        let input = document
            .create_element("input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        input.set_id(&slugify(&i.tooltip));
        input.set_type("radio");
        input.set_value(std::str::from_utf8(&vec![i.value]).unwrap());
        input.set_name("muu");
        input.set_onchange(Some(a.as_ref().unchecked_ref()));

        li.append_child(&input)?;

        let label = document
            .create_element("label")
            .unwrap()
            .dyn_into::<HtmlLabelElement>()
            .unwrap();
        label.set_html_for(&slugify(&i.tooltip));

        let span = document
            .create_element("span")
            .unwrap()
            .dyn_into::<HtmlSpanElement>()
            .unwrap();
        span.set_class_name("material-icons");
        span.set_text_content(Some(&i.icon));
        label.append_child(&span)?;
        li.append_child(&label)?;

        let span_tooltip = document
            .create_element("span")
            .unwrap()
            .dyn_into::<HtmlSpanElement>()
            .unwrap();
        span_tooltip.set_class_name("tooltip");
        span_tooltip.set_text_content(Some(&i.tooltip));
        li.append_child(&span_tooltip)?;

        ul.append_child(&li)?;
    }
    a.forget();

    let toolbar_container = document.get_element_by_id(container_id).unwrap();
    toolbar_container.append_child(&ul)?;

    Ok(())
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
            editor
                .borrow_mut()
                .mouse_move(event.client_x() as u32, event.client_y() as u32, 0, 0);
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

pub type EditorPlugin<T> = Box<dyn Plugin<T>>;

pub struct Editor<T>
where
    T: Renderer,
{
    context: Option<CanvasRenderingContext2d>,
    additional_information_layers: Vec<InformationLayer>,

    camera: Camera,
    data: T,

    plugins: Vec<EditorPlugin<T>>,

    toolbars: HashMap<ToolbarPosition, Vec<Toolbar>>,

    active_mode: u8,
    modes: HashMap<u8, Vec<Box<dyn System<T> + Send + Sync + 'static>>>,

    undo_stack: Vec<Box<dyn Action<T>>>,
    redo_stack: Vec<Box<dyn Action<T>>>,
}

impl<T> Editor<T>
where
    T: Renderer + Default,
{
    pub fn launch(&self, _width: u32, _height: u32) -> Result<(), JsValue> {
        Ok(())
    }
    pub fn new(_width: u32, _height: u32) -> Editor<T> {
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        //let (_, context) = get_canvas_and_context(&id).unwrap();
        Editor {
            context: None,
            additional_information_layers: vec![],

            plugins: Vec::new(),

            active_mode: 0,
            camera: Camera::default(),
            data: T::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            toolbars: HashMap::new(),
            modes: HashMap::new(),
        }
    }

    pub fn render(&self) -> Result<(), JsValue> {
        let context = self.context.as_ref().unwrap();

        context.clear_rect(0.0, 0.0, 2000.0, 2000.0);

        for system in self.modes.get(&self.active_mode).unwrap().iter() {
            system.render(
                &self.data,
                &context,
                &self.additional_information_layers,
                &self.plugins,
                &self.camera,
            )?;

            if system.blocks_next_systems() {
                break;
            }
        }

        Ok(())
    }

    fn mouse_pos(&self, x: u32, y: u32, camera: &Camera) -> Coordinate<f64> {
        return Coordinate {
            x: (x as i32 - camera.x).into(),
            y: (y as i32 - camera.y).into(),
        };
    }

    pub fn mouse_down(&mut self, x: u32, y: u32, button: u32) {
        // Camera control via the middle mouse button
        if button == 1 {
            self.camera.active = true;

            return;
        }

        let mouse_pos = self.mouse_pos(x, y, &self.camera);
        for system in self.modes.get_mut(&self.active_mode).unwrap().iter_mut() {
            system.mouse_down(
                mouse_pos,
                button,
                &mut self.data,
                &self.plugins,
                &mut self.undo_stack,
            );

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

        let mouse_pos = self.mouse_pos(x, y, &self.camera);
        for system in self.modes.get_mut(&self.active_mode).unwrap().iter_mut() {
            system.mouse_up(
                mouse_pos,
                button,
                &mut self.data,
                &self.plugins,
                &mut self.undo_stack,
            );

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

        let mouse_pos = self.mouse_pos(x, y, &self.camera);
        for system in self.modes.get_mut(&self.active_mode).unwrap().iter_mut() {
            system.mouse_move(
                mouse_pos,
                &mut self.data,
                &self.plugins,
                &mut self.undo_stack,
            );

            if system.blocks_next_systems() {
                break;
            }
        }
    }
}
