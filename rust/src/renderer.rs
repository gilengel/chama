use std::f64::consts::PI;

use geo::{
    Coordinate, Line, LineString, MultiLineString, MultiPoint, Point, Polygon, Rect, Triangle, GeometryCollection, MultiPolygon,
};
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

impl PrimitiveRenderer for MultiPolygon<f64> {
    fn render(&self, style: &Style, context: &CanvasRenderingContext2d) {
        for polygon in self {
            polygon.render(style, context);
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

impl PrimitiveRenderer for Line<f64> {
    fn render(&self, style: &Style, context: &CanvasRenderingContext2d) {
        context.begin_path();
        context.move_to(self.start.x, self.start.y);
        context.line_to(self.end.x, self.end.y);
        apply_style(style, context);
        context.close_path();
    }
}

impl PrimitiveRenderer for Coordinate<f64> {
    fn render(&self, style: &Style, context: &CanvasRenderingContext2d) {
        context.begin_path();
        context.arc(self.x, self.y, 5.0, 0.0, 2.0 * PI).unwrap();
        context.set_fill_style(&"#FF8C00".into());
        apply_style(style, context);
        context.close_path();
    }
}

impl PrimitiveRenderer for Point<f64> {
    fn render(&self, style: &Style, context: &CanvasRenderingContext2d) {
        context.begin_path();
        context.arc(self.x(), self.y(), 5.0, 0.0, 2.0 * PI).unwrap();
        context.set_fill_style(&"#FF8C00".into());
        apply_style(style, context);
        context.close_path();
    }
}

impl PrimitiveRenderer for MultiPoint<f64> {
    fn render(&self, style: &Style, context: &CanvasRenderingContext2d) {
        for point in self {
            point.render(style, context);
        }
    }
}

impl PrimitiveRenderer for LineString<f64> {
    fn render(&self, style: &Style, context: &CanvasRenderingContext2d) {
        let mut it = self.points_iter();

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

impl PrimitiveRenderer for MultiLineString<f64> {
    fn render(&self, style: &Style, context: &CanvasRenderingContext2d) {
        for line_string in self {
            line_string.render(style, context);
        }
    }
}

impl PrimitiveRenderer for Triangle<f64> {
    fn render(&self, style: &Style, context: &CanvasRenderingContext2d) {
        context.begin_path();
        context.move_to(self.0.x, self.0.y);
        context.move_to(self.1.x, self.1.y);
        context.move_to(self.2.x, self.2.y);   
        context.close_path();
        apply_style(style, context);
    }
}

impl PrimitiveRenderer for GeometryCollection<f64> {
    fn render(&self, style: &Style, context: &CanvasRenderingContext2d) {
        for geom in self {
            match geom {
                geo::Geometry::Point(x) => x.render(style, context),
                geo::Geometry::Line(x) => x.render(style, context),
                geo::Geometry::LineString(x) => x.render(style, context),
                geo::Geometry::Polygon(x) => x.render(style, context),
                geo::Geometry::MultiPoint(x) => x.render(style, context),
                geo::Geometry::MultiLineString(x) => x.render(style, context),
                geo::Geometry::MultiPolygon(x) => x.render(style, context),
                geo::Geometry::GeometryCollection(x) => x.render(style, context),
                geo::Geometry::Rect(x) => x.render(style, context),
                geo::Geometry::Triangle(x) => x.render(style, context),
            }
        }
    }
}