use core::result::Result;
use std::str::FromStr;

use crate::elem::ElemId;

/// Create instance via conversion from a Tiled property (`tiled::PropertyValue`).
pub trait ConvertProperty
where
    Self: Sized,
{
    /// Attempt the conversion from Tiled property.
    fn convert_property(propval: &tiled::PropertyValue) -> Result<Self, PropertyErrorKind>;
}

/// Types of errors possible when accessing/processing Tiled properties.
#[derive(Debug)]
pub enum PropertyErrorKind {
    /// The property key is not present.
    KeyNotFound,
    /// The property value (`tiled::PropertyValue`)
    UnexpectedType,
    /// String parsing/conversion failed.
    FromStrFailed,
    /// Failed to process an individual item in a list.
    ListItemParse(usize),
    /// Multiple errors occurred.
    Multi(Vec<PropertyErrorKind>),
}

/// Error returned upon failing to access/process a specific property.
#[derive(Debug)]
pub struct PropertyError {
    /// The property that was being accessed.
    pub key: String,
    /// Error code.
    pub err: PropertyErrorKind,
}

impl PropertyError {
    pub fn new(key: String, err: PropertyErrorKind) -> Self {
        Self { key, err }
    }
}

/// Retrieve the Tiled property corresponding to the key and convert it to T.
pub fn properties_get<T>(properties: &tiled::Properties, key: &str) -> Result<T, PropertyError>
where
    T: ConvertProperty,
{
    if let Some(propval) = properties.get(key) {
        T::convert_property(propval)
    } else {
        Err(PropertyErrorKind::KeyNotFound)
    }
    .map_err(|err| PropertyError {
        key: key.to_string(),
        err,
    })
}

/// Retrieve the string Tiled property corresponding to the key and parse it as a comma-separated
/// list of T.
pub fn properties_get_list<T>(
    properties: &tiled::Properties,
    key: &str,
) -> Result<Vec<T>, PropertyError>
where
    T: FromStr,
{
    let s: String = properties_get(properties, key)?;
    let items: Vec<Result<T, PropertyErrorKind>> = s
        .split(',')
        .map(|s| T::from_str(s.trim()).map_err(|_| PropertyErrorKind::FromStrFailed))
        .collect();
    let item_errors: Vec<PropertyErrorKind> = items
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            r.as_ref()
                .map_err(|_| PropertyErrorKind::ListItemParse(i))
                .err()
        })
        .collect();
    if item_errors.is_empty() {
        Ok(items.into_iter().map(|r| r.unwrap()).collect())
    } else {
        Err(PropertyError::new(
            key.to_string(),
            PropertyErrorKind::Multi(item_errors),
        ))
    }
}

impl ConvertProperty for ElemId {
    fn convert_property(propval: &tiled::PropertyValue) -> Result<Self, PropertyErrorKind> {
        match propval {
            tiled::PropertyValue::ObjectValue(o) => Ok(ElemId::Source(*o)),
            _ => Err(PropertyErrorKind::UnexpectedType),
        }
    }
}

impl ConvertProperty for f64 {
    fn convert_property(propval: &tiled::PropertyValue) -> Result<Self, PropertyErrorKind> {
        match propval {
            tiled::PropertyValue::FloatValue(x) => Ok(*x as f64),
            _ => Err(PropertyErrorKind::UnexpectedType),
        }
    }
}

impl ConvertProperty for i32 {
    fn convert_property(propval: &tiled::PropertyValue) -> Result<Self, PropertyErrorKind> {
        match propval {
            tiled::PropertyValue::IntValue(i) => Ok(*i),
            _ => Err(PropertyErrorKind::UnexpectedType),
        }
    }
}

impl ConvertProperty for String {
    fn convert_property(propval: &tiled::PropertyValue) -> Result<Self, PropertyErrorKind> {
        match propval {
            tiled::PropertyValue::StringValue(s) => Ok(s.clone()),
            _ => Err(PropertyErrorKind::UnexpectedType),
        }
    }
}

impl ConvertProperty for bool {
    fn convert_property(propval: &tiled::PropertyValue) -> Result<Self, PropertyErrorKind> {
        match propval {
            tiled::PropertyValue::BoolValue(b) => Ok(*b),
            _ => Err(PropertyErrorKind::UnexpectedType),
        }
    }
}
