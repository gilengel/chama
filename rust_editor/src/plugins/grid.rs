


use rust_macro::editor_plugin;
use web_sys::CanvasRenderingContext2d;
use super::{plugin::{Plugin, PluginWithOptions}, camera::Renderer};


#[editor_plugin]
pub struct Grid {
    #[option(default=200, min=0, max=2000, label="Offset", description="Blub blub")]
    offset: u32,

    #[option(default=4, min=0, max=100, label="Subdivisions", description="Subdivisions between offset")]
    subdivisions: u8,

    //#[option(default=true, label="Grid Enabled", description="Enables / Disables the grid")]
    //enabled: bool,
}


impl<T> Plugin<T> for Grid where T: Renderer + 'static{
    fn render(&self, context: &CanvasRenderingContext2d){  
        if self.offset == 0 {
            return;
        }

        // TODO make this dynamic
        let width = 2000.0;
        let height = 2000.0;

        context.save();
        context.set_line_width(2.0);
        context.set_stroke_style(&"rgb(40, 40, 40)".into());

        let steps_x = (width as f64 / self.offset as f64).ceil() as u32;
        let steps_y = (height as f64 / self.offset as f64).ceil() as u32;

        let sub_offset = (self.offset as f64 / self.subdivisions as f64).ceil() as u32;

        for i in 0..steps_x {
            let i = (i * self.offset).into();

            context.save();
            context.set_line_width(1.0);
            for k in 0..self.subdivisions as u32 {
                context.begin_path();
                context.move_to(i + (k * sub_offset) as f64, 0.0);
                context.line_to(i + (k * sub_offset) as f64, height.into());
                context.close_path();
                context.stroke();
            }
            context.restore();

            context.set_line_width(4.0);
            context.begin_path();
            context.move_to(i, 0.0);
            context.line_to(i, height.into());
            context.close_path();
            context.stroke();
        }

        for i in 0..steps_y {
            let i = (i * self.offset).into();

            context.save();
            context.set_line_width(1.0);
            for k in 0..self.subdivisions as u32 {
                context.begin_path();
                context.move_to(0., i + (k * sub_offset) as f64);
                context.line_to(width.into(), i + (k * sub_offset) as f64);
                context.close_path();
                context.stroke();
            }
            context.restore();

            context.set_line_width(2.0);
            context.begin_path();
            context.move_to(0.0, i);
            context.line_to(width.into(), i);
            context.close_path();
            context.stroke();
        }

        context.restore();       
    }
}