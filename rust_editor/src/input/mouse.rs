#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum Button {
    Left = 0,
    Middle = 1,
    Right = 2,
    Unknown
}

impl From<u32> for Button {
    fn from(val: u32) -> Button {
        match val {
            0 => Button::Left,
            1 => Button::Middle,
            2 => Button::Right,
            _ => Button::Unknown
        }
    }
}

impl From<i16> for Button {
    fn from(val: i16) -> Button {
        (val as u32).into()
    }
}