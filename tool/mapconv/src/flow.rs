// use core::ops::{Deref, DerefMut};
use std::str::FromStr;

// use crate::coord::*;
// use crate::elem::ElemId;

// #[derive(Debug, Clone)]
// pub struct FlowRules {
//     pub vecs: Vec<I8Vec2>,
//     pub sequence: Vec<FlowSeqCom>,
// }
//
// enum FlowVecs {
//     Extract(ElemId, Vec<f64>),
//     Convert(Vec<I8Vec2>),
//     // Magnitudes(Vec<f64>),
// }

// #[derive(Debug, Clone)]
// pub struct FlowRulesExtract {
//     pub vecs_magnitude: Vec<f64>,
//     pub sequence: Vec<FlowSeqCom>,
//     pub to: ElemId,
// }

/// Sequencer commands for the Flow effect.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlowSeqCom {
    /// Enable the flow effect.
    On,
    /// Disable the flow effect.
    Off,
    /// Use the vector at the given index (0..15) as the flow vector.
    VecIndex(u8),
    /// Prevent sequence from advancing for the given duration in 16 frame units.
    Delay(u8),
}

impl FlowSeqCom {
    pub const SEQ_A_ENABLE: u8 = 0x00;
    pub const SEQ_A_VEC: u8 = 0x10;
    pub const SEQ_A_DELAY: u8 = 0x20;

    pub fn encode_a(&self) -> u8 {
        match self {
            FlowSeqCom::On => Self::SEQ_A_ENABLE | 1,
            FlowSeqCom::Off => Self::SEQ_A_ENABLE,
            FlowSeqCom::VecIndex(i) => {
                assert!(*i < 16);
                Self::SEQ_A_VEC | *i
            }
            FlowSeqCom::Delay(d) => {
                assert!(*d < 16);
                Self::SEQ_A_DELAY | *d
            }
        }
    }
}

impl FromStr for FlowSeqCom {
    type Err = FlowSeqComParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((lhs, rhs)) = s.split_once(":") {
            match lhs {
                "v" => Ok(Self::VecIndex(rhs.parse()?)),
                "d" => Ok(Self::Delay(rhs.parse()?)),
                _ => Err(FlowSeqComParseError::NotCommandName(lhs.to_string())),
            }
        } else {
            match s {
                "on" => Ok(Self::On),
                "off" => Ok(Self::Off),
                _ => {
                    // Allow legacy plain vec index
                    let index = s.parse()?;
                    Ok(Self::VecIndex(index))
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum FlowSeqComParseError {
    InvalidFormat,
    NotCommandName(String),
    IndexOutOfRange(usize, usize),
    ParseIntError(std::num::ParseIntError),
}

impl From<std::num::ParseIntError> for FlowSeqComParseError {
    fn from(v: std::num::ParseIntError) -> Self {
        Self::ParseIntError(v)
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct FlowVecIndex(usize);
//
// impl FlowVecIndex {
//     pub const INDEX_MAX: usize = 16;
// }
//
// impl Deref for FlowVecIndex {
//     type Target = usize;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
// impl DerefMut for FlowVecIndex {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }
//
// impl FromStr for FlowVecIndex {
//     type Err = FlowSeqComParseError;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let i: usize = s.parse()?;
//         if (0..Self::INDEX_MAX).contains(&i) {
//             Ok(FlowVecIndex(i))
//         } else {
//             Err(FlowSeqComParseError::IndexOutOfRange(i, Self::INDEX_MAX))
//         }
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct FlowSeqDelayDuration(usize);
//
// impl FlowSeqDelayDuration {
//     pub const DURATION_MAX: usize = 16;
// }
//
// impl Deref for FlowSeqDelayDuration {
//     type Target = usize;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
// impl DerefMut for FlowSeqDelayDuration {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }
//
// impl FromStr for FlowSeqDelayDuration {
//     type Err = FlowSeqComParseError;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let x = s.parse()?;
//         if (0..Self::DURATION_MAX).contains(&x) {
//             Ok(FlowSeqDelayDuration(x))
//         } else {
//             Err(FlowSeqComParseError::IndexOutOfRange(x, Self::DURATION_MAX))
//         }
//     }
// }
