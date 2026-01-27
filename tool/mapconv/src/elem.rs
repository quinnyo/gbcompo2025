use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ElemId {
    /// An ID from the source map.
    Source(u32),
    /// An ID generated during conversion.
    Gen(u32),
}

impl Display for ElemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElemId::Source(value) => write!(f, "EidSrc_{:x}", value),
            ElemId::Gen(value) => write!(f, "EidGen_{:x}", value),
        }
    }
}
