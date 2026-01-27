use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
    path::PathBuf,
};

pub type TileId = Id<TileSpec>;
pub type TileSourceId = Id<TileSource>;

#[derive(Debug, Default)]
pub struct Tilesetter {
    sources: Register<TileSource>,
    tiles: Register<TileSpec>,
}

impl Tilesetter {
    pub fn register_tileset(&mut self, tileset: &tiled::Tileset) -> TileSourceId {
        self.sources.add(TileSource::from(tileset))
    }

    pub fn register_tile(&mut self, tile: &tiled::LayerTile) -> TileId {
        let source_id = self.register_tileset(tile.get_tileset());
        let index = tile.id();
        let pose = TileAtrb::from_flip_hvd(tile.flip_h, tile.flip_v, tile.flip_d);
        self.register_tile_spec(TileSpec {
            source_id,
            index,
            atrb: pose,
        })
    }

    pub fn register_tile_spec(&mut self, spec: TileSpec) -> TileId {
        self.tiles.add(spec)
    }

    pub fn get_chr_atrb(&self, tile: TileId) -> (u8, u8) {
        let spec = self.tiles.get(&tile).unwrap();
        (spec.index as u8, spec.atrb.encode())
    }

    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TileSource {
    /// Path to the source (Tiled .tsx) tileset.
    pub source: PathBuf,
}

impl TileSource {}

impl From<&tiled::Tileset> for TileSource {
    fn from(value: &tiled::Tileset) -> Self {
        TileSource {
            source: value.source.clone(),
        }
    }
}

impl RegisterItem for TileSource {
    type Item = TileSource;
    type Ids = HashIdScheme<TileSource>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TileSpec {
    /// The ID of the source tileset.
    pub source_id: TileSourceId,
    /// Location of the selected tile in the source tileset.
    pub index: u32,
    /// Override placement pose with Some(TilePose) or None to use default from tileset.
    pub atrb: TileAtrb,
}

impl RegisterItem for TileSpec {
    type Item = TileSpec;
    type Ids = HashIdScheme<TileSpec>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TileAtrb {
    pub priority: bool,
    pub flip_h: bool,
    pub flip_v: bool,
    pub flip_d: bool,
    pub bank1: bool,
    pub palette: u8,
}

impl TileAtrb {
    pub const PRIORITY: u8 = 0x80;
    pub const FLIP_V: u8 = 0x40;
    pub const FLIP_H: u8 = 0x20;
    pub const BANK: u8 = 0x08;
    pub const PALETTE: u8 = 0x07;

    pub fn from_flip_hvd(flip_h: bool, flip_v: bool, flip_d: bool) -> Self {
        Self {
            priority: false,
            flip_h,
            flip_v,
            flip_d,
            bank1: false,
            palette: 0,
        }
    }

    pub fn encode(&self) -> u8 {
        assert!(!self.flip_d, "Cannot export tile with diagonal flip.");
        assert!(self.palette <= Self::PALETTE);
        let mut a = self.palette;
        if self.bank1 {
            a |= Self::BANK;
        }
        if self.flip_h {
            a |= Self::FLIP_H;
        }
        if self.flip_v {
            a |= Self::FLIP_V;
        }
        if self.priority {
            a |= Self::PRIORITY;
        }
        a
    }
}

#[derive(Debug, Clone)]
pub struct Register<T: RegisterItem> {
    items: HashMap<Id<T>, T>,
    order: Vec<Id<T>>,
    ids: T::Ids,
}

impl<T> Register<T>
where
    T: RegisterItem<Item = T>,
{
    pub fn get(&self, id: &Id<T>) -> Option<&T> {
        self.items.get(id)
    }

    pub fn add(&mut self, item: T) -> Id<T> {
        let id = self.ids.alloc_id(&item);
        self.order.push(id.clone());
        self.items.insert(id.clone(), item);
        id
    }

    pub fn has(&self, id: &Id<T>) -> bool {
        self.items.contains_key(id)
    }
}

impl<T: RegisterItem> Default for Register<T> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            order: Default::default(),
            ids: IdScheme::new(),
        }
    }
}

/// Types that can be registered (added) to a Register.
pub trait RegisterItem: Clone {
    type Item: Clone;
    type Ids: IdScheme<Self::Item>;
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord)]
pub struct Id<T>(u64, PhantomData<T>)
where
    T: Clone;

impl<T: Clone> Id<T>
where
    T: Clone,
{
    fn new(id: u64) -> Self {
        Self(id, Default::default())
    }
}

impl<T> PartialEq for Id<T>
where
    T: Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Id<T> where T: Clone {}

impl<T> Hash for Id<T>
where
    T: Clone,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub trait IdScheme<T>
where
    T: Clone,
{
    fn alloc_id(&mut self, item: &T) -> Id<T>;

    fn new() -> Self;
}

#[derive(Debug, Default)]
pub struct HashIdScheme<T>(PhantomData<T>);

impl<T: Clone> IdScheme<T> for HashIdScheme<T>
where
    T: Hash + Eq,
{
    fn alloc_id(&mut self, item: &T) -> Id<T> {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        Id::new(hasher.finish())
    }

    fn new() -> Self {
        Self(Default::default())
    }
}
