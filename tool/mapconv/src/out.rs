use crate::{brush::Brushes, coord::*, elem::ElemId, flow};
use std::{collections::HashMap, mem};

#[derive(Debug, Default)]
pub struct Map {
    name: String,
    resources: Vec<Element>,
    chunks: Vec<Chunk>,
    /// Runtime map state allocation table.
    runtime_allocs: HashMap<ElemId, u16>,
}

impl Map {
    /// Start of `wMapState` runtime state section.
    pub const RUNTIME_STATE_ADDRESS: u16 = 0xD000;
    /// Size in bytes of `wMapState` section.
    pub const RUNTIME_STATE_SIZE: u16 = 0x400;

    pub fn rgbasm_write(&self, mut w: impl std::io::Write) -> crate::Result<()> {
        let mut code = vec![];
        self.rgbasm(&mut code);
        writeln!(&mut w, "{}", code.join("\n"))?;
        Ok(())
    }

    pub fn rgbasm(&self, code: &mut Vec<String>) {
        code.append(&mut vec![
            format!("section \"map_{}\", romx", self.name),
            format!("map_{}::", self.name),
        ]);
        let label_chunk_table = ".chunk_table";
        let label_resources = ".resources";
        code.append(&mut vec![
            format!("\tdw {}", label_chunk_table),
            format!("\tdw {}", label_resources),
        ]);
        code.append(&mut vec![
            String::default(),
            format!("{}:", label_resources),
            format!("\tdb {}", self.resources.len()),
        ]);
        for res in self.resources.iter() {
            res.rgbasm(self, code);
        }
        code.push(String::default());
        let mut chunk_table: HashMap<u8, Vec<u8>> = HashMap::new();
        for chunk in self.chunks.iter() {
            chunk.rgbasm(self, code);
            chunk_table
                .entry(chunk.coord.y)
                .or_default()
                .push(chunk.coord.x);
        }

        let mut chunk_table_rows: Vec<ChunkTableRow> = chunk_table
            .drain()
            .map(|(y, columns)| ChunkTableRow::from_y_columns(y, columns))
            .collect();

        // Sorting is not strictly required, but doing so keeps the output stable across generations.
        chunk_table_rows.sort_by(|a, b| a.y.cmp(&b.y));

        code.append(&mut vec![
            String::default(),
            format!("{}:", label_chunk_table),
            format!("\tdb ${:X}", chunk_table_rows.len()),
        ]);
        for row in chunk_table_rows.iter() {
            code.push(format!("\tdb ${:X} :: dw {}", row.y, &row.label));
        }
        for row in chunk_table_rows.iter() {
            code.push(format!("\t{}:", &row.label));
            code.push(format!("\t\tdb ${:X}", row.columns.len()));
            for x in row.columns.iter() {
                code.push(format!(
                    "\t\tdb ${:X} :: dw {}",
                    x,
                    Chunk::coord_label(*x, row.y)
                ));
            }
        }
    }

    /// Lookup the `wMapState` runtime address allocated for the element with the given id.
    pub fn runtime_address(&self, id: ElemId) -> Option<u16> {
        self.runtime_allocs.get(&id).copied()
    }

    pub fn new(name: String, resources: Vec<Element>, chunks: Vec<Chunk>) -> Self {
        let mut map = Self {
            name,
            resources,
            chunks,
            runtime_allocs: HashMap::new(),
        };
        map.sort();
        map.resolve();
        map
    }

    fn resolve(&mut self) {
        // allocate runtime memory
        let mut allocator = Allocator::new(Self::RUNTIME_STATE_ADDRESS);
        for el in self.resources.iter() {
            if let Some(size) = el.runtime_size() {
                allocator.alloc(el.id, size);
            }
        }
        self.runtime_allocs = allocator.drain().map(|it| (it.id, it.addr)).collect();
    }

    fn sort(&mut self) {
        self.resources
            .sort_by(|a, b| a.data.typeid().cmp(&b.data.typeid()).then(a.id.cmp(&b.id)));
        self.chunks
            .sort_by(|a, b| a.coord.y.cmp(&b.coord.y).then(a.coord.x.cmp(&b.coord.x)));
    }
}

struct AllocItem {
    id: ElemId,
    addr: u16,
}

#[derive(Default)]
struct Allocator {
    next: u16,
    items: Vec<AllocItem>,
}

impl Allocator {
    pub fn alloc(&mut self, id: ElemId, size: u16) -> u16 {
        let addr = self.next;
        self.next += size;
        let item = AllocItem { id, addr };
        self.items.push(item);
        addr
    }

    pub fn drain(&mut self) -> impl Iterator<Item = AllocItem> + use<'_> {
        self.next = 0;
        self.items.drain(..)
    }

    pub fn new(start: u16) -> Self {
        Self {
            next: start,
            items: Vec::new(),
        }
    }
}

struct ChunkTableRow {
    y: u8,
    label: String,
    columns: Vec<u8>,
}

impl ChunkTableRow {
    pub fn from_y_columns(y: u8, mut columns: Vec<u8>) -> Self {
        columns.sort();
        Self {
            y,
            label: format!(".row{}", y),
            columns,
        }
    }
}

pub mod code {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Code {
        /// Raw bytes
        Db(Vec<u8>),
        /// Block of Code
        Block(Vec<Code>),
        /// No-op
        Nil,
    }

    impl Code {
        /// Size of code in bytes
        pub fn sizeof(&self) -> usize {
            match self {
                Code::Db(items) => items.len(),
                Code::Block(codes) => codes.iter().map(|code| code.sizeof()).sum(),
                Code::Nil => 0,
            }
        }

        pub fn bytes(&self) -> Vec<u8> {
            match self {
                Code::Db(items) => items.clone(),
                Code::Block(codes) => codes.iter().flat_map(|code| code.bytes()).collect(),
                Code::Nil => vec![],
            }
        }
    }

    impl From<u8> for Code {
        fn from(value: u8) -> Self {
            Code::Db(vec![value])
        }
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[derive(Debug)]
    pub enum Error {
        ElementMisconfigured,
        TryFromIntError(std::num::TryFromIntError),
        Custom(&'static str),
    }

    impl From<&'static str> for Error {
        fn from(v: &'static str) -> Self {
            Self::Custom(v)
        }
    }

    impl From<std::num::TryFromIntError> for Error {
        fn from(v: std::num::TryFromIntError) -> Self {
            Self::TryFromIntError(v)
        }
    }
}

use code::Code;

/// Flow zone rules / configuration
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FlowRules {
    /// Precomputed/prescaled set of possible flow vectors.
    vecs: Vec<I8Vec2>,
    /// Flow vector selection program -- each value is an index in `vecs`
    sequence: Vec<flow::FlowSeqCom>,
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
                flow::FlowSeqCom::On => None,
                flow::FlowSeqCom::Off => None,
                flow::FlowSeqCom::VecIndex(i) => Some(i),
                flow::FlowSeqCom::Delay(_) => None,
            })
            .all(|i| (*i as usize) < self.vecs.len()));
        let vecs_code = Self::encode_array(self.vecs.iter(), |v| vec![v.x as u8, v.y as u8])?;
        let sequence_code =
            Self::encode_array(self.sequence.iter().map(flow::FlowSeqCom::encode_a), |a| {
                [a]
            })?;
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

    pub fn new(vecs: Vec<I8Vec2>, sequence: Vec<flow::FlowSeqCom>) -> Self {
        Self { vecs, sequence }
    }
}

#[derive(Debug, Default)]
pub struct OffsetTable {
    /// Target fields (sizes)
    targets: Vec<usize>,
}

impl OffsetTable {
    /// push a new target of size `sz_target` to the end of the table.
    pub fn push_target(&mut self, sz_target: usize) {
        self.targets.push(sz_target);
    }

    pub fn build<T>(&self) -> Option<Vec<T>>
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ElementType {
    /// Initial player spawn location.
    PlayerStart(U16Vec2),
    /// Tagged point
    Marker { tag: u16, position: U16Vec2 },
    /// Flow zone rules / configuration
    FlowRules(FlowRules),
    /// Zone rect. Apply rules inside rect.
    Zone {
        position: U8Vec2,
        size: U8Vec2,
        rules: ElemId,
    },
}

impl ElementType {
    pub const TYPEID_MARKER_MAX: u16 = 0x40;
    pub const TYPEID_PLAYER_START: u16 = 0x40;
    pub const TYPEID_FLOW_RULES: u16 = 0x41;
    pub const TYPEID_ZONE: u16 = 0x42;

    /// FlowState { type: db, timer: db, flow_def: dw, iseq: db, effect: db, vx: db, vy: db }
    pub const FLOW_STATE_SIZE: u16 = 1 + 1 + 2 + 1 + 1 + 1 + 1;

    /// Get the element's ("MapObject") typeid, use to identify the object in the map loader.
    pub fn typeid(&self) -> u16 {
        match self {
            ElementType::PlayerStart(_) => Self::TYPEID_PLAYER_START,
            ElementType::Marker { tag, .. } => {
                assert!(tag < &Self::TYPEID_MARKER_MAX);
                *tag
            }
            ElementType::FlowRules(_) => Self::TYPEID_FLOW_RULES,
            ElementType::Zone { .. } => Self::TYPEID_ZONE,
        }
    }

    pub fn runtime_size(&self) -> Option<u16> {
        match self {
            ElementType::PlayerStart(_) => None,
            ElementType::Marker { .. } => None,
            ElementType::FlowRules(_) => Some(Self::FLOW_STATE_SIZE),
            ElementType::Zone { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Element {
    pub id: ElemId,
    pub data: ElementType,
}

impl Element {
    pub fn rgbasm(&self, context: &Map, code: &mut Vec<String>) {
        let label = format!(".{}", self.id);
        code.append(&mut vec![
            format!("{}:", label),
            format!("\tdw ${:02X}", self.data.typeid()),
        ]);
        // runtime destination address for elements with runtime state
        if self.data.runtime_size().is_some() {
            code.push(format!(
                "\tdw ${:04X}",
                context.runtime_address(self.id).unwrap()
            ));
        }
        match self.data {
            ElementType::PlayerStart(position) => {
                code.push(format!("\tdw {}, {}", position.y, position.x))
            }
            ElementType::Marker { tag: _, position } => {
                code.push(format!("\tdw {}, {}", position.y, position.x))
            }
            ElementType::FlowRules(ref rules) => {
                code.push(format!(
                    "\tdb {}",
                    rules
                        .encode()
                        .unwrap()
                        .bytes()
                        .iter()
                        .map(|x| format!("${:02X}", *x))
                        .collect::<Vec<String>>()
                        .join(", ")
                ));
            }
            ElementType::Zone {
                position,
                size,
                rules,
            } => {
                code.append(&mut vec![
                    format!("\tdb {}, {}", position.y, size.y),
                    format!("\tdb {}, {}", position.x, size.x),
                    format!("\tdw ${:04X}", context.runtime_address(rules).unwrap()),
                ]);
            }
        }
    }

    pub fn runtime_size(&self) -> Option<u16> {
        self.data.runtime_size()
    }

    pub fn new(id: ElemId, data: ElementType) -> Self {
        Self { id, data }
    }
}

impl Ord for Element {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.id).cmp(&other.id)
    }
}

impl PartialOrd for Element {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Default)]
pub struct Chunk {
    coord: U8Vec2,
    brushes0: Brushes<u8>,
    brushes1: Brushes<u8>,
    elements: Vec<Element>,
}

impl Chunk {
    pub fn push_element(&mut self, el: Element) {
        self.elements.push(el);
    }

    pub fn rgbasm(&self, context: &Map, code: &mut Vec<String>) {
        let mut elements = self.elements.clone();
        elements.sort();

        let label = self.label();
        let label_br0 = format!("{}_br0", &label);
        let label_br1 = format!("{}_br1", &label);
        let label_elems = format!("{}_elems", &label);
        code.append(&mut vec![
            format!("{}:", &label),
            format!("\tdw {}, {}, {}", &label_br0, &label_br1, &label_elems),
        ]);
        code.push(format!("{}:", &label_br0));
        self.brushes0.rgbasm(code);
        code.push(format!("{}:", &label_br1));
        self.brushes1.rgbasm(code);
        code.append(&mut vec![
            format!("{}:", &label_elems),
            format!("\tdb {}", elements.len()),
        ]);
        for elem in elements.iter() {
            elem.rgbasm(context, code);
        }
    }

    pub fn label(&self) -> String {
        Self::coord_label(self.coord.x, self.coord.y)
    }

    pub fn new(coord: U8Vec2) -> Self {
        Self {
            coord,
            brushes0: Default::default(),
            brushes1: Default::default(),
            elements: Default::default(),
        }
    }

    pub fn with_tilemap(coord: U8Vec2, brushes0: Brushes<u8>, brushes1: Brushes<u8>) -> Self {
        Self {
            coord,
            brushes0,
            brushes1,
            elements: Default::default(),
        }
    }

    pub fn coord_label(x: u8, y: u8) -> String {
        format!(".chunk_{}_{}", x, y)
    }
}
