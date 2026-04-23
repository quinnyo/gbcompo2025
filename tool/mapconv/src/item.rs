use crate::{
    coord::*,
    out::code::Code,
    tiled_ext::{ConvertProperty, PropertyErrorKind},
};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ItemType {
    Null,
    Trash1,
    Trash3,
    Chime,
    BudAfish,
    BudSquidge,
}

impl ItemType {
    pub fn encode(&self) -> u8 {
        *self as u8
    }
}

impl From<&ItemType> for Code {
    fn from(value: &ItemType) -> Self {
        Code::Db(vec![value.encode()])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseItemTypeError;

impl FromStr for ItemType {
    type Err = ParseItemTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let slower = s.to_ascii_lowercase();
        match slower.as_str() {
            "null" => Ok(ItemType::Null),
            "trash1" => Ok(ItemType::Trash1),
            "trash3" => Ok(ItemType::Trash3),
            "chime" => Ok(ItemType::Chime),
            "bud_afish" => Ok(ItemType::BudAfish),
            "bud_squidge" => Ok(ItemType::BudSquidge),
            _ => Err(ParseItemTypeError),
        }
    }
}

impl ConvertProperty for ItemType {
    fn convert_property(propval: &tiled::PropertyValue) -> Result<Self, PropertyErrorKind> {
        match propval {
            tiled::PropertyValue::StringValue(s) => {
                ItemType::from_str(s.as_str()).map_err(|_| PropertyErrorKind::FromStrFailed)
            }
            _ => Err(PropertyErrorKind::UnexpectedType),
        }
    }
}

/// Places an instance of an item in the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPlace {
    /// Placement position in world dots
    pub position: U16Vec2,
    /// The type of item to place
    pub item: ItemType,
}

impl ItemPlace {
    pub fn new(item: ItemType, position: U16Vec2) -> Self {
        Self { position, item }
    }
}

impl Ord for ItemPlace {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.position
            .y
            .cmp(&other.position.y)
            .then(self.position.x.cmp(&other.position.x))
            .then(self.item.cmp(&other.item))
    }
}

impl PartialOrd for ItemPlace {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
