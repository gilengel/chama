use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Style {
    pub border_width: u32,
    pub border_color: String,

    pub background_color: String
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InteractiveElementStyle {
    pub normal: Style,
    pub hover: Style,
    pub selected: Style
}

impl Default for InteractiveElementStyle {
    fn default() -> Self {
        InteractiveElementStyle {
            normal: Style {
                border_width: 1,
                border_color: "#FFFFFF".to_string(),
                background_color: "#2A2A2B".to_string()
            },
            hover: Style {
                border_width: 0,
                border_color: "".to_string(),
                background_color: "#1e88e5".to_string()
            },
            selected: Style {
                border_width: 0,
                border_color: "".to_string(),
                background_color: "hsl(0, 100%, 50%)".to_string()
            },
        }
    }
}