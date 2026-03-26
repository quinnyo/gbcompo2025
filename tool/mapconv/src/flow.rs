use crate::{
    coord::*,
    out::code::{self, Code},
};
use std::mem;
use std::str::FromStr;

/// Flow zone rules / configuration
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FlowRules {
    /// Precomputed/prescaled set of possible flow vectors.
    vecs: Vec<I8Vec2>,
    /// Flow vector selection program -- each value is an index in `vecs`
    sequence: Vec<FlowSeqCom>,
}

impl FlowRules {
    /// Generate code for an array with its length (number of items) prepended.
    fn encode_array<T, U, F>(it: impl ExactSizeIterator<Item = T>, f: F) -> code::Result<Code>
    where
        U: IntoIterator<Item = u8>,
        F: Fn(T) -> U,
    {
        let n = u8::try_from(it.len())?;
        Ok(Code::Db(
            vec![n].into_iter().chain(it.flat_map(f)).collect(),
        ))
    }

    pub fn encode(&self) -> code::Result<Code> {
        assert!(!self.vecs.is_empty());
        assert!(!self.sequence.is_empty());
        assert!(self
            .sequence
            .iter()
            .filter_map(|com| match com {
                FlowSeqCom::On => None,
                FlowSeqCom::Off => None,
                FlowSeqCom::VecIndex(i) => Some(i),
                FlowSeqCom::Delay(_) => None,
            })
            .all(|i| (*i as usize) < self.vecs.len()));
        let vecs_code = Self::encode_array(self.vecs.iter(), |v| vec![v.x as u8, v.y as u8])?;
        let sequence_code =
            Self::encode_array(self.sequence.iter().map(FlowSeqCom::encode_a), |a| [a])?;
        let mut offset_table = OffsetTable::default();
        offset_table.push_target(vecs_code.sizeof());
        offset_table.push_target(sequence_code.sizeof());
        offset_table.push_target(0);

        if let Some(offsets) = offset_table.build() {
            Ok(Code::Block(vec![
                Code::Db(offsets),
                vecs_code,
                sequence_code,
            ]))
        } else {
            Err("Building offset table failed".into())
        }
    }

    pub fn new(vecs: Vec<I8Vec2>, sequence: Vec<FlowSeqCom>) -> Self {
        Self { vecs, sequence }
    }
}

#[derive(Debug, Default)]
struct OffsetTable {
    /// Target fields (sizes)
    targets: Vec<usize>,
}

impl OffsetTable {
    /// push a new target of size `sz_target` to the end of the table.
    fn push_target(&mut self, sz_target: usize) {
        self.targets.push(sz_target);
    }

    fn build<T>(&self) -> Option<Vec<T>>
    where
        T: Sized + TryFrom<usize>,
    {
        let sz_offset = mem::size_of::<T>();
        let result: Vec<T> = self
            .targets
            .iter()
            .enumerate()
            .scan(sz_offset * self.targets.len(), |addr, (i, &sz_target)| {
                let offset = *addr - (i + 1) * sz_offset;
                *addr += sz_target;
                // terminates iterator if None
                T::try_from(offset).ok()
            })
            .collect();
        if result.len() == self.targets.len() {
            Some(result)
        } else {
            None
        }
    }
}
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
