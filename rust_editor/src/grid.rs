use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

pub struct Grid {
    offset: u32,
    subdivisions: u8,
    enabled: bool,
}

impl Grid {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }

    pub fn set_offset(&mut self, offset: u32) {
        self.offset = offset;
    }

    pub fn subdivisions(&self) -> u8 {
        self.subdivisions
    }

    pub fn set_subdivisions(&mut self, subdivisions: u8) {
        self.subdivisions = subdivisions;
    }

    pub fn render(
        &self,
        context: &CanvasRenderingContext2d,
        width: u32,
        height: u32,
    ) -> Result<(), JsValue> {
        if self.offset == 0 {
            return Ok(());
        }

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

        Ok(())
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            offset: 200,
            subdivisions: 4,
            enabled: true,
        }
    }
}
