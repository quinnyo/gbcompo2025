use crate::tilemap::Tilemap;
use crate::{coord::*, tsr};

pub type ChunkTilemap = Tilemap<Option<tsr::TileId>, 16, 16>;

pub struct Chunk {
    /// Position of chunk in chunk coords
    pub position: IVec2,
    /// The tilemap data of the chunk
    pub tilemap: ChunkTilemap,
}

impl Chunk {
    pub fn new(position: IVec2) -> Self {
        Self {
            position,
            tilemap: ChunkTilemap::new(),
        }
    }
}
