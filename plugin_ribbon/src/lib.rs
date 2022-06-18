use model::ribbon_tab::RibbonTab;
use rust_editor::log;
use rust_editor::plugins::plugin::Plugin;
use rust_editor::ui::app::{EditorError, Shortkey};
use rust_macro::editor_plugin;
use wasm_bindgen::JsCast;
use web_sys::{MouseEvent, HtmlElement};
use std::collections::HashMap;
use crate::view::ribbon_tab_group::RibbonTabGroup;

pub mod model;
pub mod view;

#[editor_plugin(skip)]
pub struct RibbonPlugin<Data> {
    #[option(skip)]
    pub tabs: HashMap<&'static str, RibbonTab<Data>>,

    #[option(skip)]
    pub selected_tab: Rc<RefCell<&'static str>>
}

impl<Data> RibbonPlugin<Data> {
    pub fn get_or_add_tab(
        &mut self,
        id: &'static str,
        label: &'static str,
    ) -> Result<&RibbonTab<Data>, EditorError> {

        if !self.tabs.contains_key(id) {
            let tab = RibbonTab::new(id, label);
            self.tabs.insert(id, tab);
            
        }

        return Ok(self.tabs.get(id).unwrap()); 
    }

    pub fn get_or_add_tab_mut(
        &mut self,
        id: &'static str,
        label: &'static str,
    ) -> Result<&mut RibbonTab<Data>, EditorError> {
        self.get_or_add_tab(id, label)?;

        Ok(self.tabs.get_mut(id).unwrap())
    }
}

impl<Data> Plugin<Data> for RibbonPlugin<Data>
where
    Data: Default + 'static,
{
    fn startup(&mut self, _: &mut App<Data>) -> Result<(), EditorError> {
        *self.selected_tab.borrow_mut() = "default";

        Ok(())
    }

    fn shortkey_pressed(&mut self, _: &Shortkey, _: &Context<App<Data>>, _: &mut App<Data>) {
    }

    fn editor_elements(&mut self, ctx: &Context<App<Data>>, _: &App<Data>) -> Vec<Html> {
        use view::ribbon::Ribbon as UiRibbon;
        use view::ribbon_tab::RibbonTab as UiRibbonTab;

        let selected_tab_id = self.selected_tab.as_ref().borrow();
        let selected_tab_id: &'static str = &*selected_tab_id.borrow();

        let element = html! {

            <UiRibbon>          
                <ul class="tabs">
                {
                    html! {
                        for self.tabs.iter().map(|(id, tab)| {   
                            let selected_tab = self.selected_tab.clone();
                            let onclick = ctx.link().callback(move |e: MouseEvent| {
                                let element = e.target().unwrap().dyn_ref::<HtmlElement>().unwrap().clone();
                                let selected_tab = selected_tab.clone();
                                let mut selected_tab = selected_tab.borrow_mut();
                                *selected_tab = Box::leak(element.id().into_boxed_str());

                                EditorMessages::UpdateElements()
                            });


                            let class = if &selected_tab_id == id {
                                "selected"
                            } else {
                                ""
                            };

                            html! { <li id={&**id} class={class} {onclick}>{tab.label}</li> }
                        })
                    }
                }        
                    <hr />        
                </ul>
                <div>
                {
                    for self.tabs.iter().filter(|(id, _)| id == &&selected_tab_id).map(|(_, tab)| {                
                        html! {
                            <UiRibbonTab label={tab.label}>
                            {                        
                                for tab.groups.iter().map(|(_, group)| {
                                    html! {
                                        <RibbonTabGroup title={group.label}>
                                            {
                                                for group.actions.iter().map(|action| {
                                                    action.view(ctx) 
                                                })
                                            }
                                        </RibbonTabGroup>                                
                                    }
                                })
                            }
                            </UiRibbonTab>
                        }             
                    })
                }
                </div>
            </UiRibbon>
        };

        vec![element]
    }
}
