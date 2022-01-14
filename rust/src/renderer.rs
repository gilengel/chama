use geo::{Polygon, Rect};
use web_sys::CanvasRenderingContext2d;

use crate::style::Style;


fn apply_style(style: &Style, context: &CanvasRenderingContext2d) {
    context.set_fill_style(&style.background_color.clone().into());
    context.fill();

    if style.border_width > 0 {
        context.set_line_width(style.border_width.into());
        context.set_stroke_style(&style.border_color.clone().into());
        context.stroke();
    }    
}
pub trait PrimitiveRenderer {
    fn render(&self, style: &Style, context: &CanvasRenderingContext2d);
}

impl PrimitiveRenderer for Polygon<f64> {
    fn render(&self, style: &Style, context: &CanvasRenderingContext2d) {
        let mut it = self.exterior().points_iter();
        
        if let Some(start) = it.next() {
            context.begin_path();
            context.move_to(start.x(), start.y());
            for point in it {
                context.line_to(point.x(), point.y());
            }

            context.close_path();
            apply_style(style, context);   
        }
    }
}

impl PrimitiveRenderer for Rect<f64> {
    fn render(&self, style: &Style, context: &CanvasRenderingContext2d) {
        let min = self.min();
        let max = self.max();

        context.rect(min.x, min.y, max.x - min.x, max.y - min.y);
        apply_style(style, context);
    }
}