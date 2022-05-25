use serde::{Deserialize, Serialize};
use rand::{thread_rng, Rng};
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]

/// Defines how an element will be displayed.
pub struct Style {
    /// In pixels. A border width of 0 disables the rendering of the border.
    pub border_width: u32,

    /// Accepts all values in css format (RGB, RGBA, #000000, HSL etc.) see https://developer.mozilla.org/en-US/docs/Web/CSS/color_value
    pub border_color: String,

    /// The fill color for an element. Accepts all values in css format (RGB, RGBA, #000000, HSL etc.) see https://developer.mozilla.org/en-US/docs/Web/CSS/color_value
    pub background_color: String,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            border_width: 2,
            border_color: "#0000FF".to_string(),
            background_color: "#FFFFFFF".to_string(),
        }
    }
}

/// Defines multiple styles that can applied to an element based on the current state of it
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct InteractiveElementStyle {
    pub normal: Style,
    pub hover: Style,
    pub selected: Style,
}

impl Default for InteractiveElementStyle {
    fn default() -> Self {
        InteractiveElementStyle {
            normal: Style {
                border_width: 0,
                border_color: "#FFFFFF".to_string(),
                background_color: "#2A2A2B".to_string(),
            },
            hover: Style {
                border_width: 0,
                border_color: "".to_string(),
                background_color: "#1e88e5".to_string(),
            },
            selected: Style {
                border_width: 0,
                border_color: "".to_string(),
                background_color: "hsl(0, 100%, 50%)".to_string(),
            },
        }
    }
}

impl InteractiveElementStyle {
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let r: u8 = rng.gen_range(0..255);
        let g: u8 = rng.gen_range(0..255);
        let b: u8 = rng.gen_range(0..255);

        InteractiveElementStyle {
            normal: Style {
                border_width: 0,
                border_color: "#FFFFFF".to_string(),
                background_color: format!("rgb({},{},{})", r, g, b),
            },
            hover: Style {
                border_width: 0,
                border_color: "".to_string(),
                background_color: "#1e88e5".to_string(),
            },
            selected: Style {
                border_width: 0,
                border_color: "".to_string(),
                background_color: "hsl(0, 100%, 50%)".to_string(),
            },
        } 
    }
}