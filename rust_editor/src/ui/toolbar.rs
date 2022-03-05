use std::{rc::Rc, collections::HashMap, fmt};

use yew::{html, Context, Html, classes};


use super::{app::{App, EditorMessages, EditorError}, toolbar_button::ToolbarButton};

#[derive(Hash, PartialEq, Eq, Clone)]
pub enum ToolbarPosition {
    Left,
    Right,
    Top,
    Bottom,
}

impl fmt::Display for ToolbarPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ToolbarPosition::Left => write!(f, "left_primary_toolbar"),
            ToolbarPosition::Right => write!(f, "right_primary_toolbar"),
            ToolbarPosition::Top => write!(f, "top_primary_toolbar"),
            ToolbarPosition::Bottom => write!(f, "bottom_primary_toolbar"),
        }
    }
}

pub struct Toolbar<Data> {
    pub id: &'static str,
    pub buttons: Vec<ToolbarButton<Data>>,
}

impl<Data> Toolbar<Data> {
    pub fn add_button(
        &mut self,
        icon: &'static str,
        identifier: &'static str,
        tooltip: String,
        on_click_callback: impl Fn() -> EditorMessages<Data> + 'static,
    ) -> Result<(), EditorError> {
        let btn = ToolbarButton {
            icon,
            identifier,
            tooltip,
            on_click_callback: Rc::new(on_click_callback),
            selected: None,
        };

        self.buttons.push(btn);

        Ok(())
    }

    pub fn add_toggle_button(
        &mut self,
        icon: &'static str,
        identifier: &'static str,
        tooltip: String,
        toggled: impl Fn() -> bool + 'static,
        on_click_callback: impl Fn() -> EditorMessages<Data> + 'static,
    ) -> Result<(), EditorError> {
        let btn = ToolbarButton {
            icon,
            identifier,
            tooltip,
            on_click_callback: Rc::new(on_click_callback),
            selected: Some(Box::new(toggled)),
        };

        self.buttons.push(btn);

        Ok(())
    }
}

pub(crate) struct Toolbars<Data> {
    pub toolbars: HashMap<ToolbarPosition, Vec<Toolbar<Data>>>,
}

impl<Data> Toolbars<Data>
where
    Data:  Default,
{
    pub fn new() -> Self {
        Toolbars {
            toolbars: HashMap::new(),
        }
    }

    fn view_button(&self, button: &ToolbarButton<Data>, ctx: &Context<App<Data>>) -> Html {
        let mut classes = classes!();
        if let Some(selected_callback) = &button.selected {
            if selected_callback() {
                classes.push("selected");
            }
        }

        let callback = Rc::clone(&button.on_click_callback);
        let onclick = ctx.link().callback(move |_| (*callback)());
        html! {
            <li>
            <button onclick={onclick} class={classes}>
              <span class="material-icons">{button.icon}</span>

            </button>
            <span class="tooltip">{&button.tooltip}</span>
          </li>
        }
    }
    fn view_toolbar(&self, toolbar: &Toolbar<Data>, ctx: &Context<App<Data>>) -> Html {
        html! {
            <ul class="toolbar">
                {
                    for toolbar.buttons.iter().map(|button| {
                        self.view_button(button, ctx)
                    })
                }
            </ul>
        }
    }
    fn view_toolbars_at_pos(&self, pos: &ToolbarPosition, ctx: &Context<App<Data>>) -> Html {
        if !self.toolbars.contains_key(pos) {
            return html! {};
        }

        let id = pos.to_string();

        html! {
            <div id={id}>
            {
                for self.toolbars.get(pos).unwrap().iter().map(|toolbar| self.view_toolbar(toolbar, ctx))
            }
            </div>
        }
    }
    pub fn view(&self, ctx: &Context<App<Data>>) -> Html {
        let positions = vec![
            ToolbarPosition::Left,
            ToolbarPosition::Right,
            ToolbarPosition::Top,
            ToolbarPosition::Bottom,
        ];

        html! {
            for positions.iter().map(|pos| self.view_toolbars_at_pos(pos, ctx))
        }
    }

    pub fn add_toolbar<'a>(
        &'a mut self,
        toolbar_id: &'static str,
        position: ToolbarPosition,
    ) -> Result<&'a mut Toolbar<Data>, EditorError> {
        for (_, toolbars) in &mut self.toolbars {
            if let Some(_) = toolbars.iter_mut().find(|toolbar| toolbar.id == toolbar_id) {
                return Err(EditorError::ToolbarExists { id: toolbar_id });
            }
        }

        let toolbar = Toolbar {
            id: toolbar_id,
            buttons: vec![],
        };

        if !self.toolbars.contains_key(&position) {
            self.toolbars.insert(position.clone(), vec![toolbar]);
        } else {
            self.toolbars.get_mut(&position).unwrap().push(toolbar);
        }

        let toolbar = self
            .toolbars
            .get_mut(&position)
            .unwrap()
            .last_mut()
            .unwrap();

        Ok(toolbar)
    }

    pub fn index_and_position_of_toolbar(
        &self,
        id: &'static str,
    ) -> Result<(ToolbarPosition, usize), EditorError> {
        let mut result = self
            .toolbars
            .iter()
            .map(|(pos, toolbars)| {
                (
                    (*pos).clone(),
                    toolbars.iter().position(|toolbar| toolbar.id == id),
                )
            })
            .collect::<Vec<(ToolbarPosition, Option<usize>)>>();
        result.retain(|(_, x)| x.is_some());

        if result.is_empty() {
            return Err(EditorError::ToolbarDoesNotExists { id });
        }

        let (pos, index) = result.first().unwrap();
        let result = ((*pos).clone(), index.unwrap());
        Ok(result)
    }
}