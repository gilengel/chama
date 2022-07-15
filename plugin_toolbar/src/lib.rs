use rust_editor::log;
use rust_editor::plugin::Plugin;
use rust_editor::ui::app::{EditorError, Shortkey};
use rust_macro::editor_plugin;
use std::collections::HashMap;
use toolbar::{Toolbar, ToolbarPosition};
use yew::classes;

pub mod toolbar;
pub mod view;

pub struct ToolbarButton<Data> {
    pub icon: &'static str,
    pub identifier: &'static str,
    pub tooltip: String,
    pub on_click_callback: Rc<dyn Fn() -> EditorMessages<Data>>,
    pub selected: Option<Box<dyn Fn() -> bool>>,
  }

#[editor_plugin(skip)]
pub struct ToolbarPlugin<Data> {
    #[option(skip)]
    pub toolbars: HashMap<ToolbarPosition, Vec<Toolbar<Data>>>,
}

impl<Data> ToolbarPlugin<Data>
where
    Data: Default,
{
    pub fn add_toolbar(
        &mut self,
        toolbar_id: &'static str,
        position: ToolbarPosition,
    ) -> Result<&mut Toolbar<Data>, EditorError> {
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

    fn index_and_position_of_toolbar(
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

    pub fn get_or_add_toolbar(
        &mut self,
        toolbar_id: &'static str,
        position: ToolbarPosition,
    ) -> Result<&mut Toolbar<Data>, EditorError> {
        match self.index_and_position_of_toolbar(toolbar_id) {
            Ok((pos, index)) => Ok(self.toolbars.get_mut(&pos).unwrap().get_mut(index).unwrap()),

            Err(_) => return self.add_toolbar(toolbar_id, position),
        }
    }

    // TODO refactoring into separate yew component
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
            /*
            <span class="tooltip">{&button.tooltip}</span>
            */
          </li>
        }
    }
}

impl<Data> Plugin<Data> for ToolbarPlugin<Data>
where
    Data: Default + 'static,
{
    fn startup(&mut self, _: &mut App<Data>) -> Result<(), EditorError> {
        log!("Toolbar plugin STARTUP");

        Ok(())
    }

    fn shortkey_pressed(&mut self, _: &Shortkey, _: &Context<App<Data>>, _: &mut App<Data>) {}

    fn editor_elements(&mut self, ctx: &Context<App<Data>>, _: &App<Data>) -> Vec<Html> {
        use view::Toolbar as UiToolbar;
        //use view::ribbon_tab::RibbonTab as UiRibbonTab;

        let mut elements: Vec<Html> = vec![];
        for (pos, toolbars) in &self.toolbars {
            let id = pos.to_string();

            let element = html! {
                <div id={id}>
                {
                    for toolbars.iter().map(|toolbar| {
                        html! {
                            <UiToolbar>
                            {
                                for toolbar.buttons.iter().map(|button| {
                                    self.view_button(button, ctx)
                                })
                            }
                            </UiToolbar>
                        }
                    })
                }
                </div>
            };

            elements.push(element);
        }

        elements
    }
}
