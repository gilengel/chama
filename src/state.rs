use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{Map, Renderer};

pub trait State {
    /// Is used to implement behaviour of the state if the user clicked inside the specified
    /// html element by the statemachine.
    ///
    /// * `x` - x coordinate of the cursor where the click occured
    /// * `y` - x coordinate of the cursor where the click occured
    /// * `button` - The number of the pressed button (0=left, 1=middle, 2=right) [See here for more informations](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button)
    fn mouse_down(&mut self, x: u32, y: u32, button: u32, map: &mut Map);

    /// Is used to implement behaviour of the state if the user moved the cursor inside the
    /// specified html element by the statemaschine.
    ///
    /// * `x` - x coordinate of the cursor where the click occured
    /// * `y` - x coordinate of the cursor where the click occured
    fn mouse_move(&mut self, x: u32, y: u32, map: &mut Map);

    /// Is used to implement behaviour of the state if the user released a pressed mouse button
    /// inside the specified html element by the statemachine.
    ///
    /// * `x` - x coordinate of the cursor where the click occured
    /// * `y` - x coordinate of the cursor where the click occured
    /// * `button` - The number of the pressed button (0=left, 1=middle, 2=right) [See here for more informations](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button)
    fn mouse_up(&mut self, x: u32, y: u32, button: u32, map: &mut Map);

    fn render(&self, map: &Map, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        context.clear_rect(0.0, 0.0, map.width().into(), map.height().into());

        map.render(context)?;

        Ok(())
    }

    /// Can be used to update state variables periodically. This function is called by the
    /// state maschine within the animation loop requested by the editor.
    fn update(&mut self);

    /// Called every time the state is activated by the state maschine. Use it to 
    /// initialize values for the state.
    fn enter(&self);

    /// Called every time the state is deactivated by the state maschine. Use it to
    /// clean up values in the state.
    fn exit(&self);
}
