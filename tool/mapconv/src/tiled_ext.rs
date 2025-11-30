pub fn properties_get_int(properties: &tiled::Properties, key: &str) -> Option<i32> {
    properties
        .get(key)
        .map(|propval| match propval {
            tiled::PropertyValue::IntValue(value) => Some(*value),
            _ => None,
        })
        .flatten()
}

pub fn properties_get_bool(properties: &tiled::Properties, key: &str) -> Option<bool> {
    properties
        .get(key)
        .map(|propval| match propval {
            tiled::PropertyValue::BoolValue(value) => Some(*value),
            _ => None,
        })
        .flatten()
}
