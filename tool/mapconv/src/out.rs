use crate::{brush::Brushes, coord::*, elem::ElemId, flow::FlowRules, item::ItemPlace};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Map {
    name: String,
    resources: Vec<Element>,
    chunks: Vec<Chunk>,
    info: MapInfo,
    /// Runtime map state allocation table.
    runtime_allocs: HashMap<ElemId, u16>,
}

impl Map {
    /// Start of `wMapState` runtime state section.
    pub const RUNTIME_STATE_ADDRESS: u16 = 0xD000;
    /// Size in bytes of `wMapState` section.
    pub const RUNTIME_STATE_SIZE: u16 = 0x400;
    /// Maximum number of trash items that can be placed in a map.
    pub const TRASH_ITEMS_COUNT_MAX: u8 = 200;
    /// Size of trash item collection state in bytes.
    pub const TRASH_ITEMS_BYTES: u8 = Self::TRASH_ITEMS_COUNT_MAX.div_ceil(8);

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
        let label_info = ".info";
        let label_chunk_table = ".chunk_table";
        let label_resources = ".resources";
        code.append(&mut vec![
            format!("\tdw {}", label_info),
            format!("\tdw {}", label_chunk_table),
            format!("\tdw {}", label_resources),
        ]);

        // info
        code.append(&mut vec![
            String::default(),
            format!("{}:", label_info),
            format!("\tdb {}", self.info.trash_item_count),
        ]);

        // resources
        code.append(&mut vec![
            String::default(),
            format!("{}:", label_resources),
            format!("\tdb {}", self.resources.len()),
        ]);
        for res in self.resources.iter() {
            res.rgbasm(self, code);
        }

        // chunks
        code.push(String::default());
        let mut chunk_table: HashMap<u8, Vec<u8>> = HashMap::new();
        for chunk in self.chunks.iter() {
            chunk.rgbasm(self, code);
            chunk_table
                .entry(chunk.coord.y)
                .or_default()
                .push(chunk.coord.x);
        }

        // chunk table
        let mut chunk_table_rows: Vec<ChunkTableRow> = chunk_table
            .drain()
            .map(|(y, columns)| ChunkTableRow::from_y_columns(y, columns))
            .collect();

        // Sorting is not strictly required, but doing so keeps the output stable across generations.
        chunk_table_rows.sort_by_key(|a| a.y);

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

    pub fn new(name: String, resources: Vec<Element>, chunks: Vec<Chunk>, info: MapInfo) -> Self {
        let mut map = Self {
            name,
            resources,
            chunks,
            info,
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

#[derive(Debug, Default)]
pub struct MapInfo {
    /// Number of trash items placed in the map.
    pub trash_item_count: u8,
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
        /// An array of 8 bit bytes. Equivalent to `DB` in rgbasm.
        Db(Vec<u8>),
        /// An array of 16 bit words. Equivalent to `DW` in rgbasm.
        Dw(Vec<u16>),
        /// A block of Code
        Block(Vec<Code>),
        Pin,
        /// No-op
        Nil,
    }

    impl Code {
        /// Size of code in bytes
        pub fn sizeof(&self) -> usize {
            match self {
                Code::Db(items) => items.len(),
                Code::Dw(items) => items.len() * 2,
                Code::Block(codes) => codes.iter().map(|code| code.sizeof()).sum(),
                Code::Pin => 0,
                Code::Nil => 0,
            }
        }

        pub fn bytes(&self) -> Vec<u8> {
            match self {
                Code::Db(items) => items.clone(),
                Code::Dw(items) => items.iter().flat_map(|x| x.to_le_bytes()).collect(),
                Code::Block(codes) => codes.iter().flat_map(|code| code.bytes()).collect(),
                Code::Pin => vec![],
                Code::Nil => vec![],
            }
        }

        pub fn rgbasm(&self, lines: &mut Vec<String>) {
            match self {
                Code::Db(items) => {
                    lines.push(format!(
                        "\tdb {}",
                        items
                            .iter()
                            .map(|x| format!("${:02X}", *x))
                            .collect::<Vec<String>>()
                            .join(", ")
                    ));
                }
                Code::Dw(items) => {
                    lines.push(format!(
                        "\tdw {}",
                        items
                            .iter()
                            .map(|x| format!("${:04X}", *x))
                            .collect::<Vec<String>>()
                            .join(", ")
                    ));
                }
                Code::Block(codes) => {
                    for code in codes.iter() {
                        code.rgbasm(lines);
                    }
                }
                Code::Pin => todo!(),
                Code::Nil => todo!(),
            }
        }
    }

    impl From<u8> for Code {
        fn from(value: u8) -> Self {
            Code::Db(vec![value])
        }
    }

    impl From<u16> for Code {
        fn from(value: u16) -> Self {
            Code::Dw(vec![value])
        }
    }

    impl From<&[u8]> for Code {
        fn from(value: &[u8]) -> Self {
            Code::Db(value.into())
        }
    }

    impl From<&[u16]> for Code {
        fn from(value: &[u16]) -> Self {
            Code::Dw(value.into())
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

    /**
    FlowState {
        +1 | type: db,
        +1 | timer: db,
        +2 | flow_def: dw,
    =    4
        +1 | iseq: db,
        +1 | effect: db,
        +1 | vx: db,
        +1 | vy: db,
    =    8
        +1 | sprite_count: db,
        +2 | sprite: dw,
    =   11
    }
    **/
    pub const FLOW_STATE_SIZE: u16 = 11;

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
    items: Vec<ItemPlace>,
}

impl Chunk {
    /// Maximum number of items in one chunk.
    pub const ITEMS_MAX: usize = 16;

    /// Chunk origin in world dots.
    pub fn origin(&self) -> U16Vec2 {
        self.coord.as_u16vec2() * 8 * 16
    }

    pub fn push_element(&mut self, el: Element) {
        self.elements.push(el);
    }

    pub fn push_item(&mut self, itp: ItemPlace) {
        assert!(
            self.items.len() < Self::ITEMS_MAX,
            "Cannot add item to Chunk<{}, {}>, items array is full.",
            self.coord.x,
            self.coord.y
        );
        self.items.push(itp);
    }

    pub fn rgbasm(&self, context: &Map, code: &mut Vec<String>) {
        let mut elements = self.elements.clone();
        elements.sort();

        let label = self.label();
        let label_br0 = format!("{}_br0", &label);
        let label_br1 = format!("{}_br1", &label);
        let label_elems = format!("{}_elems", &label);
        let label_items = format!("{}_items", &label);
        code.append(&mut vec![
            format!("{}:", &label),
            format!(
                "\tdw {}, {}, {}, {}",
                &label_br0, &label_br1, &label_elems, &label_items
            ),
        ]);
        code.push(format!("{}:", &label_br0));
        self.brushes0.rgbasm(code);
        code.push(format!("{}:", &label_br1));
        self.brushes1.rgbasm(code);
        // elements
        code.append(&mut vec![
            format!("{}:", &label_elems),
            format!("\tdb {}", elements.len()),
        ]);
        for elem in elements.iter() {
            elem.rgbasm(context, code);
        }
        // items
        let mut items = self.items.clone();
        items.sort();
        code.append(&mut vec![
            format!("{}:", &label_items),
            format!("\tdb {} ; items.len()", items.len()),
        ]);
        let chunk_origin = self.origin();
        for itp in items.iter() {
            let position = itp.position - chunk_origin;
            assert!(
                position.x < 128 && position.y < 128,
                "Chunk item must be inside chunk"
            );
            let x = position.x as u8;
            let y = position.y as u8;
            let item_type = itp.item.encode();
            code.append(&mut vec![format!(
                "\t\tdb {}, {}, {}, {} ; item {}: {:?}",
                itp.id, y, x, item_type, itp.id, itp.item
            )]);
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
            items: Default::default(),
        }
    }

    pub fn with_tilemap(coord: U8Vec2, brushes0: Brushes<u8>, brushes1: Brushes<u8>) -> Self {
        Self {
            coord,
            brushes0,
            brushes1,
            elements: Default::default(),
            items: Default::default(),
        }
    }

    pub fn coord_label(x: u8, y: u8) -> String {
        format!(".chunk_{}_{}", x, y)
    }
}
