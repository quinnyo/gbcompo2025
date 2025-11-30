use crate::coord::C2i32;
use crate::tiled_ext;

/// Object collector manager & do thinger
pub struct Objects {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectType {
    /// Player spawn point
    PlayerStart,
    /// A reference point
    Marker(u8),
    /// Unrecognised user type string
    UserType(String),
    /// Probably pointless untyped object type
    Void,
}

impl TryFrom<&tiled::Object<'_>> for ObjectType {
    type Error = &'static str;

    fn try_from(value: &tiled::Object) -> Result<Self, Self::Error> {
        match value.user_type.as_str() {
            "PlayerStart" => Ok(Self::PlayerStart),
            "Marker" => {
                if let Some(id) = tiled_ext::properties_get_int(&value.properties, "id") {
                    Ok(Self::marker(id as u8))
                } else {
                    Err("no 'id' on Marker object")
                }
            }
            s => {
                if s.is_empty() {
                    Ok(Self::Void)
                } else {
                    Ok(Self::UserType(value.user_type.clone()))
                }
            }
        }
    }
}

impl ObjectType {
    pub const ITEM_ID_MAX: u8 = 64;

    pub fn marker(id: u8) -> Self {
        assert!(id < Self::ITEM_ID_MAX);
        Self::Marker(id)
    }

    pub fn encode(&self) -> Option<u8> {
        match self {
            ObjectType::PlayerStart => Some(Self::ITEM_ID_MAX),
            ObjectType::Marker(id) => {
                assert!(*id < Self::ITEM_ID_MAX);
                Some(*id)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    pub id: u32,
    pub position: C2i32,
    pub shape: tiled::ObjectShape,
    pub properties: tiled::Properties,
    pub data: ObjectType,
}

impl TryFrom<tiled::Object<'_>> for Object {
    type Error = &'static str;

    fn try_from(value: tiled::Object) -> Result<Self, Self::Error> {
        let data = ObjectType::try_from(&value)?;
        Ok(Self {
            id: value.id(),
            position: C2i32::new(value.x.round() as i32, value.y.round() as i32),
            shape: value.shape.clone(),
            properties: value.properties.clone(),
            data,
        })
    }
}
