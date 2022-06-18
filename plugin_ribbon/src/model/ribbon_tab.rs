use rust_editor::ui::app::EditorError;

use super::ribbon_tab_group::RibbonTabGroup;
use std::collections::HashMap;

pub struct RibbonTab<Data> {
    pub id: &'static str,
    pub label: &'static str,

    pub groups: HashMap<&'static str, RibbonTabGroup<Data>>,
}

impl<Data> RibbonTab<Data> {
    pub fn new(id: &'static str, label: &'static str) -> Self {
        RibbonTab {
            id,
            label,
            groups: HashMap::new(),
        }
    }

    pub fn get_or_add_group(
        &mut self,
        id: &'static str,
        label: &'static str,
    ) -> Result<&RibbonTabGroup<Data>, EditorError> {

        if !self.groups.contains_key(id) {
            let group = RibbonTabGroup::new(id, label);
            self.groups.insert(id, group);
            
        }

        return Ok(self.groups.get(id).unwrap()); 
    }

    pub fn get_or_add_group_mut(
        &mut self,
        id: &'static str,
        label: &'static str,
    ) -> Result<&mut RibbonTabGroup<Data>, EditorError> {

        if !self.groups.contains_key(id) {
            let group = RibbonTabGroup::new(id, label);
            self.groups.insert(id, group);
            
        }

        return Ok(self.groups.get_mut(id).unwrap()); 
    }
}
