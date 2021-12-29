use crate::intersection::Intersection;

pub struct Street<'a> {
    pub start: &'a Intersection,
    pub end: &'a Intersection
}

impl<'a> Street<'a> {
    pub fn length(&self) -> f32 {
        (self.start.position - self.end.position).length()
    }

    pub fn new(start: &'a Intersection, end: &'a Intersection) -> Street<'a> {
        Street {
            start,
            end
        }
    }

    /*
    pub fn set_start(&mut self, start: &'a Intersection) {
        self.start.remove_outgoing_street(self);

        self.start = start;
    }

    pub fn set_end(&mut self, end: &'a Intersection) {
        self.end.remove_incoming_street(self);

        self.end = end;
        self.end.add_incoming_street(self);
        self.start.add_outgoing_street(self);
    }
    */
}