use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    World,
    Ui,
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Layer::World => write!(f, "World"),
            Layer::Ui => write!(f, "Ui"),
        }
    }
}
