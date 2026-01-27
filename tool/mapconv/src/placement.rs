use crate::{coord::*, elem::ElemId, geometry::Shape, tiled_ext};

/// Extracted editor object. Places a something in the map.
#[derive(Debug, Clone)]
pub struct Placement {
    pub id: ElemId,
    pub name: String,
    pub position: IVec2,
    pub shape: Shape,
    pub properties: tiled::Properties,
    pub data: PlacementType,
}

impl TryFrom<&tiled::Object<'_>> for Placement {
    type Error = PlacementTypeError;

    fn try_from(value: &tiled::Object) -> Result<Self, Self::Error> {
        let data = PlacementType::try_from(value)?;
        Ok(Self {
            id: ElemId::Source(value.id()),
            name: value.name.clone(),
            position: IVec2::new(value.x.round() as i32, value.y.round() as i32),
            shape: From::from(&value.shape),
            properties: value.properties.clone(),
            data,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlacementType {
    /// Player spawn point
    PlayerStart,
    /// A reference point
    Marker(u8),
    /// Effective area for a Zone
    ZArea { modulate: f64, rules: ElemId },
    /// Flow Zone rules
    ZFlow { magnitude: f64, to: ElemId },
    /// Unrecognised user type string
    Unknown(String),
}

impl TryFrom<&tiled::Object<'_>> for PlacementType {
    type Error = PlacementTypeError;

    fn try_from(value: &tiled::Object) -> Result<Self, Self::Error> {
        match value.user_type.as_str() {
            "PlayerStart" => Ok(Self::PlayerStart),
            "Marker" => {
                if let Some(id) = tiled_ext::properties_get_int(&value.properties, "id") {
                    Ok(Self::Marker(id as u8))
                } else {
                    Err(PlacementTypeError::Property {
                        user_type: "Marker",
                        property: "id",
                    })
                }
            }
            "ZArea" => {
                if let Some(rules) = tiled_ext::properties_get_object(&value.properties, "rules") {
                    Ok(Self::ZArea {
                        modulate: tiled_ext::properties_get_float(&value.properties, "modulate")
                            .unwrap_or(1.0),
                        rules,
                    })
                } else {
                    Err(PlacementTypeError::Property {
                        user_type: "ZArea",
                        property: "rules",
                    })
                }
            }
            "ZFlow" => {
                if let Some(magnitude) =
                    tiled_ext::properties_get_float(&value.properties, "magnitude")
                {
                    if let Some(to) = tiled_ext::properties_get_object(&value.properties, "to") {
                        Ok(Self::ZFlow { magnitude, to })
                    } else {
                        Err(PlacementTypeError::Property {
                            user_type: "ZFlow",
                            property: "to",
                        })
                    }
                } else {
                    Err(PlacementTypeError::Property {
                        user_type: "ZFlow",
                        property: "magnitude",
                    })
                }
            }
            s => Ok(Self::Unknown(s.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum PlacementTypeError {
    Failed,
    Property {
        user_type: &'static str,
        property: &'static str,
    },
}
