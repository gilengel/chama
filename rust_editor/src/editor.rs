use std::{cell::RefCell, collections::HashMap, panic, rc::Rc, sync::{MutexGuard, Mutex}};

use geo::Coordinate;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue, UnwrapThrowExt};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, HtmlLabelElement, HtmlSpanElement, HtmlElement, HtmlSelectElement};

use crate::{
    actions::Action,
    camera::{Camera, Renderer},
    html, html_impl, log,
    macros::slugify,
    system::System,
    InformationLayer,
};

/*
#[function_component(Toolbar)]
pub fn toolbar() -> Html {
    html! {
        <p>
            { "TOOLBAR" }
        </p>

    }
}
*/

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
    pub icon: String,
    pub tooltip: String,
    pub value: String,
}

pub fn add_toolbar<T>(mut editor: MutexGuard<'static, Editor<T>>, toolbar: Toolbar, position: ToolbarPosition) -> Result<(), JsValue> where T: Renderer + Default + 'static  {

    let container_id = match position {
        ToolbarPosition::Top => "top_primary_toolbar",
        ToolbarPosition::Right => "right_primary_toolbar",
        ToolbarPosition::Bottom => "bottom_primary_toolbar",
        ToolbarPosition::Left => "left_primary_toolbar",
    };

    if editor.toolbars.get(&position).is_none() {
        editor.toolbars.insert(position.clone(), vec![]);

        html! {
            ["main"]

            <div id=(container_id)></div>
        }
    }

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let ul = document
        .create_element("ul")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();

    ul.set_class_name("toolbar");
    ul.set_attribute("role", "radiogroup")?;

    let a = Closure::wrap(Box::new(move |evt: web_sys::Event| {
        log!("{}", evt.target().unwrap().dyn_into::<HtmlInputElement>().unwrap().value());
    }) as Box<dyn Fn(_)>);

    for i in &toolbar.buttons {
        let li = document.create_element("li").unwrap().dyn_into::<HtmlElement>().unwrap();
        let input = document.create_element("input").unwrap().dyn_into::<HtmlInputElement>().unwrap();
        input.set_id(&slugify(&i.tooltip));
        input.set_type("radio");
        input.set_value(&i.value);
        input.set_name("muu");
        input.set_onchange(Some(a.as_ref().unchecked_ref()));
        
        
        li.append_child(&input)?;

        let label = document.create_element("label").unwrap().dyn_into::<HtmlLabelElement>().unwrap();
        label.set_html_for(&slugify(&i.tooltip));

        let span = document.create_element("span").unwrap().dyn_into::<HtmlSpanElement>().unwrap();
        span.set_class_name("material-icons");
        span.set_text_content(Some(&i.icon));
        label.append_child(&span)?;
        li.append_child(&label)?;

        let span_tooltip = document.create_element("span").unwrap().dyn_into::<HtmlSpanElement>().unwrap();
        span_tooltip.set_class_name("tooltip");
        span_tooltip.set_text_content(Some(&i.tooltip));
        li.append_child(&span_tooltip)?;
        
        ul.append_child(&li)?;
    }
    a.forget();

    let toolbar_container = document.get_element_by_id(container_id).unwrap();
    toolbar_container.append_child(&ul)?;
    /*
    html! {
        [container_id]

        <div class="toolbar" role="radiogroup">
        (for i in &toolbar.buttons) {
            <li>
                <input id=(slugify(&i.tooltip)) type="radio" value=(i.value) name="muu"></input>

                <label for=(slugify(&i.tooltip))>
                    <span class="material-icons">(i.icon)</span>
                </label>
                <span class="tooltip">(i.tooltip)</span>
            </li>
        }
       </div>
    }
    */

    //editor.toolbars.get_mut(&position).unwrap().push(toolbar);

    Ok(())

    /*
    let window = web_sys::window().expect("should have a window in this context");
    let document = window.document().expect("window should have a document");

    let num_clicks = document
        .get_element_by_id("num-clicks")
        .expect("should have #num-clicks on the page");
    let mut clicks = 0;

    let a = Closure::wrap(Box::new(move || {
        
        editor.muu();
        
        clicks += 1;
        num_clicks.set_inner_html(&clicks.to_string());
    }) as Box<dyn FnMut()>);
    document
        .get_element_by_id("green-square")
        .expect("should have #green-square on the page")
        .dyn_ref::<HtmlElement>()
        .expect("#green-square be an `HtmlElement`")
        .set_onclick(Some(a.as_ref().unchecked_ref()));

    // See comments in `setup_clock` above for why we use `a.forget()`.
    a.forget();
    */
}

pub struct Editor<T>
where
    T: Renderer,
{
    //context: CanvasRenderingContext2d,
    additional_information_layers: Vec<InformationLayer>,

    active_systems: Vec<Box<dyn System<T> + Send + Sync>>,
    camera: Camera,
    data: T,

    toolbars: HashMap<ToolbarPosition, Vec<Toolbar>>,

    modes: Vec<String>,

    undo_stack: Vec<Box<dyn Action<T>>>,
    redo_stack: Vec<Box<dyn Action<T>>>,
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

unsafe impl<T> Sync for Editor<T> where T: Renderer {}

impl<T> Editor<T>
where
    T: Renderer + Default,
{

    pub fn launch(&self) {
        let window = web_sys::window().expect("global window does not exists");
        let document = window.document().expect("expecting a document on window");
        let body = document
            .body()
            .expect("document expect to have have a body");

        let button = document.create_element("button").unwrap();
        button.set_text_content(Some("Bella Ciao"));

        let editor = Rc::new(RefCell::new(self));
        {
            let editor = editor.clone();

            let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                //let a = editor.borrow_mut();
                //a.muuh();
            }) as Box<dyn FnMut(_)>);
            button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        body.append_child(&button);
    }
    pub fn new(width: u32, height: u32) -> Editor<T> {
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        //let (_, context) = get_canvas_and_context(&id).unwrap();
        Editor {
            //context,
            additional_information_layers: vec![],

            active_systems: Vec::new(),
            camera: Camera::default(),
            data: T::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            toolbars: HashMap::new(),
            modes: Vec::new(),
        }
    }

    pub fn deactivate_system(&mut self) {
        todo!();
    }

    pub fn deactivate_all_systems(&mut self) {
        self.active_systems.clear();
    }

    pub fn render(&self) -> Result<(), JsValue> {
        /*
        self.context
            .clear_rect(0.0, 0.0, 2000.0, 2000.0);

        for system in &self.active_systems {
            system.render(
                &self.data,
                &self.context,
                &self.additional_information_layers,
                &self.camera,
            )?;

            if system.blocks_next_systems() {
                break;
            }
        }
        */

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
        for system in &mut self.active_systems {
            system.mouse_down(mouse_pos, button, &mut self.data, &mut self.undo_stack);

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
        for system in &mut self.active_systems {
            system.mouse_up(mouse_pos, button, &mut self.data, &mut self.undo_stack);

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
        for system in &mut self.active_systems {
            system.mouse_move(mouse_pos, &mut self.data, &mut self.undo_stack);

            if system.blocks_next_systems() {
                break;
            }
        }
    }
}
