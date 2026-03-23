use std::str::FromStr;

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
