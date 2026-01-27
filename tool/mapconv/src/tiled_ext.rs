use crate::elem::ElemId;

pub fn properties_get_int(properties: &tiled::Properties, key: &str) -> Option<i32> {
    properties.get(key).and_then(|propval| match propval {
        tiled::PropertyValue::IntValue(value) => Some(*value),
        _ => None,
    })
}

pub fn properties_get_bool(properties: &tiled::Properties, key: &str) -> Option<bool> {
    properties.get(key).and_then(|propval| match propval {
        tiled::PropertyValue::BoolValue(value) => Some(*value),
        _ => None,
    })
}

pub fn properties_get_float(properties: &tiled::Properties, key: &str) -> Option<f64> {
    properties.get(key).and_then(|propval| match propval {
        tiled::PropertyValue::FloatValue(value) => Some(*value as f64),
        _ => None,
    })
}

pub fn properties_get_string(properties: &tiled::Properties, key: &str) -> Option<String> {
    properties.get(key).and_then(|propval| match propval {
        tiled::PropertyValue::StringValue(value) => Some(value.clone()),
        _ => None,
    })
}

pub fn properties_get_object(properties: &tiled::Properties, key: &str) -> Option<ElemId> {
    properties.get(key).and_then(|propval| match propval {
        tiled::PropertyValue::ObjectValue(value) => Some(ElemId::Source(*value)),
        _ => None,
    })
}
