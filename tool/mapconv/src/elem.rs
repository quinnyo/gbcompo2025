use std::fmt::Display;

use crate::extract::ExtractNodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ElemId {
    /// An ID from the source map.
    Source(u32),
    /// An ID generated during conversion.
    Gen(u32),
    /// Temporary/transition patch for interop with extract IDs...
    ExtractId(ExtractNodeId),
}

impl Display for ElemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElemId::Source(value) => write!(f, "EidSrc_{:x}", value),
            ElemId::Gen(value) => write!(f, "EidGen_{:x}", value),
            ElemId::ExtractId(id) => match id {
                ExtractNodeId::Root => write!(f, "XidRoot"),
                ExtractNodeId::Layer(value) => write!(f, "XidLayer_{:x}", value),
                ExtractNodeId::Object(value) => write!(f, "XidObject_{:x}", value),
            },
        }
    }
}
