use model::ribbon::Ribbon;
use model::ribbon_action::RibbonAction;
use model::ribbon_tab::RibbonTab;
use model::ribbon_tab_group::RibbonTabGroup;
use rust_editor::plugin::Plugin;
use rust_editor::ui::app::{EditorError, Shortkey};
use rust_macro::editor_plugin;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, MouseEvent};
use yew::virtual_dom::VNode;

pub mod model;
pub mod view;

#[editor_plugin(skip)]
pub struct RibbonPlugin<Data> {
    #[option(skip)]
    pub tabs: HashMap<&'static str, RibbonTab<Data>>,

    #[option(skip)]
    pub selected_tab: Rc<RefCell<&'static str>>,
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

impl<Data> RibbonPlugin<Data>
where
    Data: Default,
{
    fn view_ribbon(&self, ribbon: &Ribbon<Data>, ctx: &Context<App<Data>>) -> VNode {
        let selected_tab_id = *self.selected_tab.as_ref().borrow();
        html! {
            <div class="ribbon">
                <ul class="tabs">
                {

                        for ribbon.tabs.iter().map(|tab| {
                            let selected_tab = self.selected_tab.clone();
                            let onclick = ctx.link().callback(move |e: MouseEvent| {
                                let element = e.target().unwrap().dyn_ref::<HtmlElement>().unwrap().clone();
                                let selected_tab = selected_tab.clone();
                                let mut selected_tab = selected_tab.borrow_mut();
                                *selected_tab = Box::leak(element.id().into_boxed_str());

                                EditorMessages::UpdateElements()
                            });


                            let class = if selected_tab_id == tab.id {
                                "selected"
                            } else {
                                ""
                            };

                            html! { <li id={selected_tab_id} class={class} {onclick}>{tab.label}</li> }
                        })

                }
                    <hr />
                </ul>
                <div>
                {
                    for self.tabs.iter().filter(|(id, _)| id == &&selected_tab_id).map(|(_, tab)| {
                        self.view_ribbon_tab(tab, ctx)
                        /*
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
                        */
                    })
                }
                </div>
            </div>
        }
    }

    fn view_ribbon_group(&self, group: &RibbonTabGroup<Data>, ctx: &Context<App<Data>>) -> VNode {
        html! {
        <div class="ribbon_tab_group">
            <div class="content">
            {
                for group.actions.iter().map(|action| {
                    self.view_ribbon_action(action, ctx)
                })
            }
            </div>
            <span>{&group.label}</span>
        </div>
        }
    }

    fn view_ribbon_action(
        &self,
        action: &Box<dyn RibbonAction<Data>>,
        ctx: &Context<App<Data>>,
    ) -> VNode {
        action.view(ctx)
    }

    fn view_ribbon_tab(&self, tab: &RibbonTab<Data>, ctx: &Context<App<Data>>) -> VNode {
        html! {
            <div class="ribbon_tab">
            {
                for tab.groups.iter().map(|(_, group)| {
                    self.view_ribbon_group(group, ctx)
                })
            }
            </div>
        }
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

    fn shortkey_pressed(&mut self, _: &Shortkey, _: &Context<App<Data>>, _: &mut App<Data>) {}

    fn editor_elements(&mut self, ctx: &Context<App<Data>>, _: &App<Data>) -> Vec<Html> {

        let selected_tab_id = *self.selected_tab.as_ref().borrow();
        let element =         
        html! {
            <div class="ribbon">
                <ul class="tabs">
                {
                    for self.tabs.iter().map(|(_, tab)| {
                        let selected_tab = self.selected_tab.clone();
                        let onclick = ctx.link().callback(move |e: MouseEvent| {
                            let element = e.target().unwrap().dyn_ref::<HtmlElement>().unwrap().clone();
                            let selected_tab = selected_tab.clone();
                            let mut selected_tab = selected_tab.borrow_mut();
                            *selected_tab = Box::leak(element.id().into_boxed_str());

                            EditorMessages::UpdateElements()
                        });


                        let class = if selected_tab_id == tab.id {
                            "selected"
                        } else {
                            ""
                        };
                        
                        html! { <li id={selected_tab_id} class={class} {onclick}>{tab.label}</li> }
                    })
                }
                    <hr />
                </ul>
                <div>
                {
                    for self.tabs.iter().filter(|(id, _)| id == &&selected_tab_id).map(|(_, tab)| {
                        self.view_ribbon_tab(tab, ctx)
                    })
                }
                </div>
            </div>
        };

        vec![element]
    }
}
