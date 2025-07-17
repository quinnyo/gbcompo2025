use std::{cmp, fmt, io, ops};

const PROP_EDITOR_ONLY: &str = "editor_only";

#[derive(Debug, Default)]
pub struct BgAttributes {
    pub priority: bool,
    pub flip_y: bool,
    pub flip_x: bool,
    pub bank1: bool,
    pub palette: u8,
}

impl BgAttributes {
    pub const PRIORITY: u8 = 0x80;
    pub const FLIP_Y: u8 = 0x40;
    pub const FLIP_X: u8 = 0x20;
    pub const BANK: u8 = 0x08;
    pub const PALETTE: u8 = 0x07;

    pub fn encode_bin(&self) -> u8 {
        assert!(self.palette <= Self::PALETTE);
        let mut a = self.palette;
        if self.bank1 {
            a |= Self::BANK;
        }
        if self.flip_x {
            a |= Self::FLIP_X;
        }
        if self.flip_y {
            a |= Self::FLIP_Y;
        }
        if self.priority {
            a |= Self::PRIORITY;
        }
        a
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub struct C2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy + Ord> C2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn min_each(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    pub fn max_each(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }
}

impl<T> From<(T, T)> for C2<T> {
    fn from(src: (T, T)) -> Self {
        Self { x: src.0, y: src.1 }
    }
}

impl<T: ops::Add<Output = T>> ops::Add for C2<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub for C2<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Ord> Ord for C2<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.y.cmp(&other.y).then(self.x.cmp(&other.x))
    }
}

impl<T: Ord> PartialOrd for C2<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord + fmt::Display> fmt::Display for C2<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}, {}>", self.x, self.y)
    }
}

pub trait Rgbasm {
    fn rgbasm(&self, w: impl io::Write) -> Result<(), io::Error>;
}

impl Rgbasm for u8 {
    fn rgbasm(&self, mut w: impl io::Write) -> Result<(), io::Error> {
        write!(&mut w, "{:3}", self)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Brush<T> {
    Solid { rep: usize, val: T },
    Terminator,
}

impl<T> Brush<T> {
    pub const BR_SOLID: &str = "BR_SOLID";
    pub const BR_TERM: &str = "BR_TERM";

    pub fn terminator(&self) -> bool {
        match self {
            Brush::Terminator => true,
            _ => false,
        }
    }

    /// Number of tiles emitted by this brush.
    pub fn size(&self) -> usize {
        match self {
            Brush::Solid { rep, val: _val } => *rep + 1,
            Brush::Terminator => 0,
        }
    }

    pub fn from_datum(val: T) -> Self {
        Self::Solid { rep: 0, val }
    }
}

impl<T: Rgbasm> Rgbasm for Brush<T> {
    fn rgbasm(&self, mut w: impl io::Write) -> Result<(), io::Error> {
        match self {
            Brush::Solid { rep, val } => {
                write!(&mut w, "db {}, {:3}, ", Self::BR_SOLID, rep)?;
                val.rgbasm(w)?;
            }
            Brush::Terminator => {
                write!(&mut w, "db {}", Self::BR_TERM)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ChunkBrushes<T> {
    data: Vec<Brush<T>>,
}

impl<T: Eq + PartialEq> ChunkBrushes<T> {
    pub fn last(&self) -> Option<&Brush<T>> {
        self.data.last()
    }

    pub fn last_mut(&mut self) -> Option<&mut Brush<T>> {
        self.data.last_mut()
    }

    pub fn push_literal(&mut self, pushee: T) {
        if let Some(brush) = self.last_mut() {
            match brush {
                Brush::Solid { rep, val } => {
                    if *val == pushee {
                        *rep += 1;
                        return;
                    }
                }
                Brush::Terminator => panic!(),
            }
        }
        self.push(Brush::from_datum(pushee));
    }

    pub fn push(&mut self, b: Brush<T>) {
        assert!(!self.is_terminated());
        self.data.push(b);
    }

    pub fn is_terminated(&self) -> bool {
        self.last().is_some_and(|b| b.terminator())
    }

    /// number of brushes
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// number of tiles
    pub fn size(&self) -> usize {
        self.data.iter().map(|b| b.size()).sum()
    }

    pub fn brushes(&self) -> impl Iterator<Item = &Brush<T>> {
        self.data.iter()
    }
}

impl<T> From<ChunkBrushes<T>> for Vec<Brush<T>> {
    fn from(mut brushes: ChunkBrushes<T>) -> Vec<Brush<T>> {
        std::mem::take(&mut brushes.data)
    }
}

pub type C2i32 = C2<i32>;
pub type ChunkCoord = u8;
pub type ChunkIndex = u8;
pub type TileChr = u8;
pub type TileAtrb = u8;

const MAP_SOURCE_PREFIX: &str = "src/assets/maps";

#[derive(Debug, Default)]
pub struct MapConverter {
    pub map_name: Box<String>,
    min_pos: C2i32,
    max_pos: C2i32,
    chunks: Vec<(C2i32, ChunkBrushes<TileChr>, ChunkBrushes<TileAtrb>)>,
}

impl MapConverter {
    /// Bounding size of the map in chunk coordinates.
    pub fn dim(&self) -> C2i32 {
        self.max_pos - self.min_pos
    }

    /// Returns the number of chunks in the map.
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn sort(&mut self) {
        self.chunks.sort_by(|a, b| a.0.cmp(&b.0));
    }

    /// Returns an iterator over the map chunks, with normalised coordinates.
    pub fn normalised(
        &self,
    ) -> impl Iterator<Item = (C2i32, &ChunkBrushes<TileChr>, &ChunkBrushes<TileAtrb>)> {
        self.chunks
            .iter()
            .map(|(pos, chrs, atrbs)| (*pos - self.min_pos, chrs, atrbs))
    }

    /// Extract & convert a Tiled TMX map.
    pub fn process_tmx(&mut self, tmx: tiled::Map) -> Result<(), io::Error> {
        assert!(tiled::ChunkData::HEIGHT == 16);
        assert!(tiled::ChunkData::WIDTH == 16);
        assert!(tmx.infinite());
        assert!(self.chunks.is_empty());
        if self.map_name.is_empty() {
            let name = tmx
                .source
                .strip_prefix(MAP_SOURCE_PREFIX)
                .unwrap_or(tmx.source.as_path())
                .with_extension("");
            self.map_name = Box::new(name.to_str().unwrap().to_owned());
        }
        self.min_pos = Default::default();
        self.max_pos = Default::default();
        for layer in tmx.layers() {
            if Self::tiled_properties_get_bool(&layer.properties, PROP_EDITOR_ONLY) {
                continue;
            } else {
                self.process_layer(layer);
            }
        }
        self.sort();
        Ok(())
    }

    fn process_layer(&mut self, layer: tiled::Layer) {
        match layer.layer_type() {
            tiled::LayerType::Tiles(tile_layer) => match tile_layer {
                tiled::TileLayer::Infinite(inf) => {
                    for (chunk_pos, chunk) in inf.chunks() {
                        self.process_chunk(chunk_pos.into(), chunk);
                    }
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    fn process_chunk(&mut self, pos: C2i32, chunk: tiled::Chunk) {
        self.min_pos = self.min_pos.min_each(pos);
        self.max_pos = self.max_pos.max_each(pos);
        let brushes = Self::tiled_chunk_extract_brushes(chunk);
        self.chunks.push((pos, brushes.0, brushes.1));
    }

    fn tiled_chunk_extract_brushes(
        src_chunk: tiled::Chunk,
    ) -> (ChunkBrushes<TileChr>, ChunkBrushes<TileAtrb>) {
        let mut chr_brushes: ChunkBrushes<TileChr> = Default::default();
        let mut atrb_brushes: ChunkBrushes<TileAtrb> = Default::default();
        for y in 0..tiled::ChunkData::HEIGHT as i32 {
            for x in 0..tiled::ChunkData::WIDTH as i32 {
                if let Some(layer_tile) = src_chunk.get_tile(x, y) {
                    assert!(layer_tile.id() < 256);
                    chr_brushes.push_literal(layer_tile.id() as TileChr);
                    assert!(layer_tile.flip_d == false);
                    atrb_brushes.push_literal(
                        BgAttributes {
                            flip_y: layer_tile.flip_v,
                            flip_x: layer_tile.flip_h,
                            ..Default::default()
                        }
                        .encode_bin(),
                    );
                } else {
                    break;
                }
            }
        }
        chr_brushes.push(Brush::Terminator);
        atrb_brushes.push(Brush::Terminator);
        (chr_brushes, atrb_brushes)
    }

    fn tiled_properties_get_bool(properties: &tiled::Properties, key: &str) -> bool {
        if let Some(propval) = properties.get(key) {
            match propval {
                tiled::PropertyValue::BoolValue(value) => *value,
                _ => false,
            }
        } else {
            false
        }
    }
}

impl Rgbasm for MapConverter {
    fn rgbasm(&self, mut w: impl io::Write) -> Result<(), io::Error> {
        assert!(!self.map_name.is_empty());

        let mut chunk_idx = 0;
        let mut chunk_table: Vec<(ChunkCoord, Vec<(ChunkCoord, ChunkIndex)>)> = vec![];

        writeln!(&mut w, "include \"map.rgbinc\"\n")?;
        writeln!(&mut w, "section \"map_{}\", romx", self.map_name)?;
        writeln!(&mut w, "map_{}::", self.map_name)?;

        writeln!(&mut w, "\tdw .chunk_table")?;
        writeln!(&mut w, "\t; dw .other_stuff")?;

        for (pos, chrs, atrbs) in self.normalised() {
            assert!(atrbs.size() == chrs.size());
            let ntiles = chrs.size();
            writeln!(&mut w, ".chunk_{chunk_idx}: ; {pos} ({ntiles})")?;
            for brush in chrs.brushes() {
                write!(&mut w, "\t")?;
                brush.rgbasm(&mut w)?;
                writeln!(&mut w)?;
            }

            assert!(pos.x < 256);
            assert!(pos.y < 256);
            let (x, y) = (pos.x as ChunkCoord, pos.y as ChunkCoord);

            let entry = (x, chunk_idx);
            if chunk_table.last().is_some_and(|row| row.0 == y) {
                let (_last_y, last_data) = chunk_table.last_mut().unwrap();
                last_data.push(entry);
            } else {
                chunk_table.push((y, vec![entry]));
            }
            chunk_idx += 1;
        }

        // print chunk table
        writeln!(&mut w, ".chunk_table: db {}", chunk_table.len())?;
        for (y, data) in chunk_table.iter() {
            writeln!(&mut w, "\tdb {}, {}", y, data.len())?;
            for (x, idx) in data.iter() {
                writeln!(&mut w, "\t\tdb {x}\n\t\tdw .chunk_{idx}")?;
            }
        }

        Ok(())
    }
}
