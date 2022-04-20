#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum Key {
    Left = 0,
    Middle = 1,
    Right = 2,
    Unknown
}

impl From<u32> for Key {
    fn from(val: u32) -> Key {
        match val {
            0 => Key::Left,
            1 => Key::Middle,
            2 => Key::Right,
            _ => Key::Unknown
        }
    }
}

impl From<i16> for Key {
    fn from(val: i16) -> Key {
        (val as u32).into()
    }
}