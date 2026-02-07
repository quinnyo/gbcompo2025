use std::collections::HashMap;

use crate::{
    brush::{Brush, Brushes},
    chunk,
    coord::*,
    elem::ElemId,
    geometry::Shape,
    out::{self, Element, ElementType, FlowRules, Map},
    placement::{Placement, PlacementType},
    tsr::Tilesetter,
};
use chunk::ChunkTilemap;

pub struct Builder {
    /// The map's name/identifier.
    pub name: String,
    /// Tileset buildmanager
    pub tiles: Tilesetter,
    /// Chunks by position in chunk coords. Chunks subdivide the map space.
    pub chunks: HashMap<IVec2, chunk::Chunk>,

    pub placements: HashMap<ElemId, Placement>,
}

impl Builder {
    pub fn extract_layer(&mut self, layer: &tiled::Layer<'_>) {
        match layer.layer_type() {
            tiled::LayerType::Tiles(tile_layer) => match tile_layer {
                tiled::TileLayer::Finite(_) => panic!("only infinite tile layers are supported"),
                tiled::TileLayer::Infinite(inf) => {
                    assert!(
                        tiled::ChunkData::HEIGHT as usize == ChunkTilemap::DIM_Y
                            && tiled::ChunkData::WIDTH as usize == ChunkTilemap::DIM_X
                    );
                    for ((src_x, src_y), src_chunk) in inf.chunks() {
                        let mut tilemap = ChunkTilemap::new();
                        for y in 0..tiled::ChunkData::HEIGHT as usize {
                            for x in 0..tiled::ChunkData::WIDTH as usize {
                                if let Some(tile) = src_chunk.get_tile(x as i32, y as i32) {
                                    tilemap.xy_set(x, y, Some(self.tiles.register_tile(&tile)));
                                }
                            }
                        }
                        let chunk = chunk::Chunk {
                            position: IVec2::new(src_x, src_y),
                            tilemap,
                        };
                        self.chunks.insert(chunk.position, chunk);
                    }
                }
            },
            tiled::LayerType::Objects(object_layer) => {
                for o in object_layer.objects() {
                    let mut placement = Placement::try_from(&o).expect("failed to convert object");
                    placement.position +=
                        IVec2::new(layer.offset_x.round() as i32, layer.offset_y.round() as i32);
                    self.placements.insert(placement.id, placement);
                }
            }
            _ => panic!(),
        }
    }

    /// Convert/compile/build Map
    pub fn build(&self) -> Map {
        assert!(!self.chunks.is_empty());
        let mut chunk_coords: Vec<IVec2> = self.chunks.keys().cloned().collect();
        chunk_coords.sort_by(|a, b| a.y.cmp(&b.y).then(a.x.cmp(&b.x)));
        let chunk_origin: IVec2 = chunk_coords
            .iter()
            .cloned()
            .reduce(|a, b| a.min(b))
            .unwrap_or_default();
        let dot_origin = chunk_origin * 16 * 8;

        // Convert tilemaps/chunks
        let mut chunks: HashMap<U8Vec2, out::Chunk> = chunk_coords
            .iter()
            .map(|old_coord| {
                let mut chr_brushes: Brushes<u8> = Default::default();
                let mut atrb_brushes: Brushes<u8> = Default::default();
                for (_, _, cell) in self.chunks.get(old_coord).unwrap().tilemap.iter() {
                    let (chr, atrb) = cell
                        .clone()
                        .map(|tile| self.tiles.get_chr_atrb(tile))
                        .unwrap_or((0, 0));
                    chr_brushes.push_literal(chr);
                    atrb_brushes.push_literal(atrb);
                }
                chr_brushes.push(Brush::Terminator);
                atrb_brushes.push(Brush::Terminator);
                let coord = (old_coord - chunk_origin).as_u8vec2();
                (
                    coord,
                    out::Chunk {
                        coord,
                        brushes0: chr_brushes,
                        brushes1: atrb_brushes,
                        elements: Vec::new(),
                    },
                )
            })
            .collect();

        // Convert placements
        let mut resources: Vec<Element> = Vec::new();
        for placement in self.placements.values() {
            let id = placement.id;
            let position = (placement.position - dot_origin).as_u16vec2();

            match placement.data {
                PlacementType::PlayerStart => {
                    resources.push(Element::new(id, ElementType::PlayerStart(position)))
                }
                PlacementType::Marker(id) => resources.push(Element::new(
                    placement.id,
                    ElementType::Marker {
                        tag: id as u16,
                        position,
                    },
                )),
                PlacementType::ZArea { modulate: _, rules } => {
                    if let Shape::Rect { size } = placement.shape {
                        assert!(placement.shape.has_area());
                        // TODO: handle zones that straddle chunks in some way.
                        let chunk_pos = position / 8 / 16;
                        let local_position = position - chunk_pos * 16 * 8;
                        let chunk = chunks
                            .entry(chunk_pos.as_u8vec2())
                            .or_insert_with(|| out::Chunk::new(chunk_pos.as_u8vec2()));
                        chunk.elements.push(Element::new(
                            id,
                            ElementType::Zone {
                                position: local_position.as_u8vec2(),
                                size: size.floor().as_u8vec2(),
                                rules,
                            },
                        ));
                    } else {
                        panic!("expected Rect shape for Zone");
                    }
                }
                PlacementType::ZFlow {
                    ref vecs_magnitude,
                    ref sequence,
                    to,
                } => {
                    if let Some(rhs) = self.placements.get(&to) {
                        let diff = rhs.position.as_dvec2() - placement.position.as_dvec2();
                        let dir = diff
                            .try_normalize()
                            .expect("failed to normalise flow vector");
                        let vecs = vecs_magnitude
                            .iter()
                            .map(|m| (m * dir).round().as_i8vec2())
                            .collect();

                        resources.push(Element::new(
                            id,
                            ElementType::FlowRules(FlowRules::new(vecs, sequence.clone())),
                        ));
                    } else {
                        panic!("ZFlow 'to' object not found");
                    }
                }
                PlacementType::Unknown(ref s) => {
                    eprintln!(
                        "Cannot convert placement <'{}':{}> with unknown type ('{}')",
                        placement.name, placement.id, s
                    );
                }
            }
        }

        Map::new(
            self.name.clone(),
            resources,
            chunks.drain().map(|(_, chunk)| chunk).collect(),
        )
    }

    /// Create a new, empty, Builder.
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            tiles: Tilesetter::new(),
            chunks: HashMap::new(),
            placements: HashMap::new(),
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
