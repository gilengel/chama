use std::{cell::RefCell, rc::Rc};

use uuid::Uuid;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{HtmlElement, HtmlInputElement, HtmlLabelElement, HtmlSpanElement};

use crate::{
    editor::{document, slugify, Editor},
    plugins::{self, camera::Renderer},
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ToolbarPosition {
    Top,
    Right,
    Bottom,
    Left,
}

pub struct Toolbar<T>
where
    T: Renderer + 'static,
{
    pub buttons: Vec<ToolbarButton<T>>,
    pub position: ToolbarPosition,

    id: String,
}

impl<T> Toolbar<T>
where
    T: Renderer + 'static,
{
    pub fn new(buttons: Vec<ToolbarButton<T>>, position: ToolbarPosition) -> Self {
        Toolbar {
            buttons,
            position,

            id: Uuid::new_v4().to_string(),
        }
    }
    pub fn render(&self, editor: Rc<RefCell<Editor<T>>>) -> Result<(), JsValue>
    where
        T: plugins::camera::Renderer + 'static,
    {
        let container_id = match self.position {
            ToolbarPosition::Top => "top_primary_toolbar",
            ToolbarPosition::Right => "right_primary_toolbar",
            ToolbarPosition::Bottom => "bottom_primary_toolbar",
            ToolbarPosition::Left => "left_primary_toolbar",
        };

        let document = document();
        let ul = document
            .create_element("ul")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();

        ul.set_id(&self.id);
        ul.set_class_name("toolbar");
        ul.set_attribute("role", "radiogroup")?;

        for button in &self.buttons {
            ul.append_child(&button.render(editor.clone()).unwrap())?;
        }

        let container = match document.get_element_by_id(container_id) {
            Some(e) => e,
            None => {
                let container = document.create_element("div").unwrap();
                container.set_id(container_id);

                document
                    .get_element_by_id("main")
                    .unwrap()
                    .append_child(&container)?;
                container
            }
        };
        container.append_child(&ul)?;

        Ok(())
    }

    pub fn add_button(&mut self, button: ToolbarButton<T>, editor: Rc<RefCell<Editor<T>>>) -> Result<(), JsValue> {
        let ul = document().get_element_by_id(&self.id).unwrap();

        ul.append_child(&button.render(editor).unwrap())?;

        self.buttons.push(button);

        Ok(())
    }
}

pub struct ToolbarButton<T>
where
    T: Renderer + 'static,
{
    pub icon: &'static str,
    pub tooltip: &'static str,
    pub value: u8,

    pub callback: fn(Rc<RefCell<Editor<T>>>),
}

impl<T> ToolbarButton<T>
where
    T: Renderer,
{
    pub fn new(
        icon: &'static str,
        tooltip: &'static str,
        value: u8,
        callback: fn(Rc<RefCell<Editor<T>>>),
    ) -> Self {
        ToolbarButton {
            icon,
            tooltip,
            value,
            callback,
        }
    }

    pub fn render(&self, editor: Rc<RefCell<Editor<T>>>) -> Result<HtmlElement, JsValue>
    {
        let document = document();

        let id = slugify(&self.tooltip);

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
        input.set_id(&id);
        input.set_type("radio");
        input.set_value(std::str::from_utf8(&vec![self.value]).unwrap());
        input.set_name("muu");

        let func = self.callback.clone();
        // Callback
        let a = Closure::wrap(Box::new(move |evt: web_sys::Event| {
            func(editor.clone());

        }) as Box<dyn FnMut(_)>);
        input.set_onchange(Some(a.as_ref().unchecked_ref()));
        a.forget();

        li.append_child(&input)?;

        let label = document
            .create_element("label")
            .unwrap()
            .dyn_into::<HtmlLabelElement>()
            .unwrap();
        label.set_html_for(&slugify(&self.tooltip));

        let span = document
            .create_element("span")
            .unwrap()
            .dyn_into::<HtmlSpanElement>()
            .unwrap();
        span.set_class_name("material-icons");
        span.set_text_content(Some(&self.icon));
        label.append_child(&span)?;
        li.append_child(&label)?;

        let span_tooltip = document
            .create_element("span")
            .unwrap()
            .dyn_into::<HtmlSpanElement>()
            .unwrap();
        span_tooltip.set_class_name("tooltip");
        span_tooltip.set_text_content(Some(&self.tooltip));
        li.append_child(&span_tooltip)?;

        Ok(li)
    }
}
