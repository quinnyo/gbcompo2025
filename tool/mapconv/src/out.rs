use crate::{brush::Brushes, coord::*, elem::ElemId};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Map {
    pub name: String,
    pub resources: Vec<Element>,
    pub chunks: Vec<Chunk>,
}

impl Map {
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
            res.rgbasm(code);
        }
        code.push(String::default());
        let mut chunk_table: HashMap<u8, Vec<u8>> = HashMap::new();
        for chunk in self.chunks.iter() {
            chunk.rgbasm(code);
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
}

struct ChunkTableRow {
    y: u8,
    label: String,
    columns: Vec<u8>,
}

impl ChunkTableRow {
    pub fn from_y_columns(y: u8, columns: Vec<u8>) -> Self {
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
    pub const TYPEID_MARKER_MAX: u16 = 64;
    pub const TYPEID_PLAYER_START: u16 = 64;
    pub const TYPEID_FLOW: u16 = 65;
    pub const TYPEID_ZONE: u16 = 66;

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
}

#[derive(Debug)]
pub struct Element {
    pub id: ElemId,
    pub data: ElementType,
}

impl Element {
    pub fn rgbasm(&self, code: &mut Vec<String>) {
        let label = format!(".{}", self.id);
        code.append(&mut vec![
            format!("{}:", label),
            format!("dw ${:02X}", self.data.typeid()),
        ]);
        match self.data {
            ElementType::PlayerStart(position) => {
                code.push(format!("dw {}, {}", position.y, position.x))
            }
            ElementType::Marker { tag: _, position } => {
                code.push(format!("dw {}, {}", position.y, position.x))
            }
            ElementType::Flow(v) => code.push(format!("db {}, {}", v.x, v.y)),
            ElementType::Zone { .. } => (),
        }
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
    pub fn rgbasm(&self, code: &mut Vec<String>) {
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
