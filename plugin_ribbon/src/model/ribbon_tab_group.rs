use super::ribbon_action::RibbonAction;

pub struct RibbonTabGroup<Data> {
    pub id: &'static str,
    pub label: &'static str,

    pub actions: Vec<Box<dyn RibbonAction<Data> + 'static>>
}

impl<Data> RibbonTabGroup<Data> {
    pub fn new(id: &'static str, label: &'static str) -> Self {
        RibbonTabGroup {
            id,
            label,
            actions: Vec::new()
        }
    }

    pub fn add_action<T>(&mut self, action: T) where T : RibbonAction<Data> + 'static {
        self.actions.push(Box::new(action))
    }
}