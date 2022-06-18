use super::ribbon_tab::RibbonTab;

pub struct Ribbon<Data> {
    pub id: &'static str,
    pub tabs: Vec<RibbonTab<Data>>,
}

impl<Data> Ribbon<Data> {

}
