use crate::{coord::*, elem::ElemId, flow, geometry::Shape, tiled_ext};

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
    type Error = UserTypeError;

    fn try_from(value: &tiled::Object) -> Result<Self, Self::Error> {
        let data = PlacementType::try_from(value).map_err(|err| UserTypeError {
            user_type: value.user_type.to_string(),
            err,
        })?;
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
    ZFlow {
        vecs_magnitude: Vec<f64>,
        sequence: Vec<flow::FlowSeqCom>,
        to: ElemId,
    },
    /// Unrecognised user type string
    Unknown(String),
}

impl TryFrom<&tiled::Object<'_>> for PlacementType {
    type Error = UserTypeErrorKind;

    fn try_from(value: &tiled::Object) -> Result<Self, Self::Error> {
        match value.user_type.as_str() {
            "PlayerStart" => Ok(Self::PlayerStart),
            "Marker" => {
                let id: i32 = tiled_ext::properties_get(&value.properties, "id")?;
                Ok(Self::Marker(id as u8))
            }
            "ZArea" => {
                let rules = tiled_ext::properties_get(&value.properties, "rules")?;
                let modulate =
                    tiled_ext::properties_get(&value.properties, "modulate").unwrap_or(1.0);
                Ok(Self::ZArea { modulate, rules })
            }
            "ZFlow" => {
                let vecs_magnitude =
                    tiled_ext::properties_get_list(&value.properties, "vecs_magnitude")?;
                let sequence = tiled_ext::properties_get_list(&value.properties, "sequence")?;
                let to = tiled_ext::properties_get(&value.properties, "to")?;
                Ok(PlacementType::ZFlow {
                    vecs_magnitude,
                    sequence,
                    to,
                })
            }
            s => Ok(Self::Unknown(s.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum UserTypeErrorKind {
    Property(tiled_ext::PropertyError),
}

impl From<tiled_ext::PropertyError> for UserTypeErrorKind {
    fn from(v: tiled_ext::PropertyError) -> Self {
        Self::Property(v)
    }
}

/// Error converting a user type from Tiled.
#[derive(Debug)]
pub struct UserTypeError {
    pub user_type: String,
    pub err: UserTypeErrorKind,
}
