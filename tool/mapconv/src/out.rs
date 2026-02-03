use crate::{brush::Brushes, coord::*, elem::ElemId};
use std::collections::HashMap;

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
            res.rgbasm(&self, code);
        }
        code.push(String::default());
        let mut chunk_table: HashMap<u8, Vec<u8>> = HashMap::new();
        for chunk in self.chunks.iter() {
            chunk.rgbasm(&self, code);
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

#[derive(Debug)]
pub enum ElementType {
    /// Initial player spawn location.
    PlayerStart(U16Vec2),
    /// Tagged point
    Marker { tag: u16, position: U16Vec2 },
    /// Flow Zone rules resource. Flow vector is directional force.
    Flow(I8Vec2),
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
    pub const TYPEID_FLOW: u16 = 0x41;
    pub const TYPEID_ZONE: u16 = 0x42;

    /// Get the element's ("MapObject") typeid, use to identify the object in the map loader.
    pub fn typeid(&self) -> u16 {
        match self {
            ElementType::PlayerStart(_) => Self::TYPEID_PLAYER_START,
            ElementType::Marker { tag, .. } => {
                assert!(tag < &Self::TYPEID_MARKER_MAX);
                *tag
            }
            ElementType::Flow(_) => Self::TYPEID_FLOW,
            ElementType::Zone { .. } => Self::TYPEID_ZONE,
        }
    }

    pub fn runtime_size(&self) -> Option<u16> {
        match self {
            ElementType::PlayerStart(_) => None,
            ElementType::Marker { .. } => None,
            ElementType::Flow(_) => Some(2),
            ElementType::Zone { .. } => None,
        }
    }
}

#[derive(Debug)]
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
            ElementType::Flow(v) => code.push(format!("\tdb {}, {}", v.x, v.y)),
            ElementType::Zone {
                position,
                size,
                rules,
            } => {
                code.append(&mut vec![
                    format!("\tdb {}, {}", position.y, position.x),
                    format!("\tdb {}, {}", size.y, size.x),
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

#[derive(Debug, Default)]
pub struct Chunk {
    pub coord: U8Vec2,
    pub brushes0: Brushes<u8>,
    pub brushes1: Brushes<u8>,
    pub elements: Vec<Element>,
}

impl Chunk {
    pub fn rgbasm(&self, context: &Map, code: &mut Vec<String>) {
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
            format!("\tdb {}", self.elements.len()),
        ]);
        for elem in self.elements.iter() {
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

    pub fn coord_label(x: u8, y: u8) -> String {
        format!(".chunk_{}_{}", x, y)
    }
}
