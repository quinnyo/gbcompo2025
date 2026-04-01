use crate::{
    coord::*,
    out::code::Code,
    tiled_ext::{ConvertProperty, PropertyErrorKind},
};

/// Places a Doodad (a simple object) at a specified position in the world.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DoodadPlace {
    pub doodad: Doodad,
    pub option: u8,
    pub position: U16Vec2,
}

impl DoodadPlace {
    /// Minimal Doodad placement at the given point.
    pub fn at_point(doodad: Doodad, position: U16Vec2) -> Self {
        Self {
            doodad,
            option: 0,
            position,
        }
    }
}

impl From<&DoodadPlace> for Code {
    fn from(value: &DoodadPlace) -> Self {
        let U16Vec2 { x, y } = value.position;
        Code::Block(vec![
            value.doodad.into(),
            value.option.into(),
            Code::Dw(vec![y, x]),
        ])
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Doodad {
    /// Specify a doodad using its internal ID code directly. This value is not guaranteed to
    /// correspond to a defined Doodad.
    Code(u8),
}

impl Doodad {
    pub fn encode(&self) -> u8 {
        match self {
            Doodad::Code(i) => *i,
        }
    }
}

impl ConvertProperty for Doodad {
    fn convert_property(propval: &tiled::PropertyValue) -> Result<Self, PropertyErrorKind> {
        match propval {
            tiled::PropertyValue::IntValue(i) => Ok(Doodad::from(i.rem_euclid(256) as u8)),
            _ => Err(PropertyErrorKind::UnexpectedType),
        }
    }
}

impl From<u8> for Doodad {
    fn from(value: u8) -> Self {
        Doodad::Code(value)
    }
}

impl From<Doodad> for Code {
    fn from(value: Doodad) -> Self {
        Code::from(value.encode())
    }
}
