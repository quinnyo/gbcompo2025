use std::{collections::HashMap, path::PathBuf};

use crate::{
    chunk::{self, ChunkTilemap},
    coord::*,
    geometry,
    tiled_ext::{ConvertProperty, PropertyErrorKind},
    tsr::Tilesetter,
};

/// Complete map extraction context & extracted data.
#[derive(Debug, Default, PartialEq)]
pub struct Extract {
    pub map_info: ExtractMapInfo,
    pub nodes: HashMap<ExtractNodeId, ExtractNode>,
    pub tiles: Tilesetter,
    pub chunks: HashMap<IVec2, chunk::Chunk>,
    /// Top-level layers
    toplevel: Vec<ExtractNodeId>,
}

impl Extract {
    /// Determine whether or not `trunk` is an ancestor of `leaf`. Returns true if `leaf` descends
    /// from `trunk`.
    pub fn descends_from(&self, leaf: ExtractNodeId, trunk: ExtractNodeId) -> bool {
        let mut search_id = self.get_parent_id(leaf);
        while search_id.is_some() && search_id != Some(trunk) {
            search_id = self.get_parent_id(search_id.unwrap());
        }
        search_id == Some(trunk)
    }

    /// If a node with ID `child_id` is present, then retrieve the node's parent and return the
    /// parent's ID. If the `child_id` node was not found, return `None`.
    pub fn get_parent_id(&self, child_id: ExtractNodeId) -> Option<ExtractNodeId> {
        self.nodes.get(&child_id).map(|node| node.parent_id)
    }

    /// Access the extracted node with the given ID.
    pub fn access(&self, id: &ExtractNodeId) -> Option<ExtractNodeAccess<'_>> {
        self.nodes
            .get(id)
            .map(|node| ExtractNodeAccess::new(self, node))
    }

    /// Create a new extraction context and extract the map data from a Tiled map.
    pub fn extract_tmx(tmx: &tiled::Map) -> Self {
        let mut extract = Extract {
            map_info: ExtractMapInfo::from_map(tmx),
            ..Default::default()
        };
        extract.extract_layers(tmx.layers());
        extract
    }

    fn insert_node(&mut self, node: ExtractNode) {
        if node.parent_id == ExtractNodeId::Root {
            self.toplevel.push(node.id);
        }
        self.nodes.insert(node.id, node);
    }

    fn extract_layers<'a, I>(&mut self, it: I)
    where
        I: IntoIterator<Item = tiled::Layer<'a>>,
    {
        // (parent_id, child)
        let mut queue: Vec<(ExtractNodeId, tiled::Layer<'_>)> = it
            .into_iter()
            .map(|layer| (ExtractNodeId::Root, layer))
            .collect();

        while !queue.is_empty() {
            let mut next: Vec<(ExtractNodeId, tiled::Layer<'_>)> = vec![];
            for (parent_id, layer) in queue.drain(..) {
                let id = ExtractNodeId::Layer(layer.id());
                let mut object_ids: Vec<ExtractNodeId> = vec![];
                match layer.layer_type() {
                    tiled::LayerType::Tiles(tile_layer) => match tile_layer {
                        tiled::TileLayer::Finite(_) => {
                            panic!("only infinite tile layers are supported")
                        }
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
                                            tilemap.xy_set(
                                                x,
                                                y,
                                                Some(self.tiles.register_tile(&tile)),
                                            );
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
                        for obnode in object_layer
                            .objects()
                            .map(|ob| Self::extract_object(id, ob))
                        {
                            object_ids.push(obnode.id);
                            self.insert_node(obnode);
                        }
                    }
                    tiled::LayerType::Image(_) => {
                        eprintln!("Image layer is not supported.");
                        continue;
                    }
                    tiled::LayerType::Group(group) => {
                        next.extend(group.layers().map(|sub| (id, sub)));
                    }
                }
                if let Some(parent) = self.nodes.get_mut(&parent_id) {
                    parent.child_ids.push(id);
                }
                self.insert_node(ExtractNode {
                    id,
                    user_type: layer.user_type.to_owned().unwrap_or_default(),
                    properties: layer.properties.clone(),
                    offset: DVec2::new(layer.offset_x as f64, layer.offset_y as f64),
                    visible: layer.visible,
                    parent_id,
                    child_ids: vec![],
                    object_ids,
                    shape: None,
                });
            }
            queue.append(&mut next);
        }
    }

    fn extract_object(parent_id: ExtractNodeId, ob: tiled::Object<'_>) -> ExtractNode {
        ExtractNode {
            id: ExtractNodeId::Object(ob.id()),
            user_type: ob.user_type.clone(),
            properties: ob.properties.clone(),
            offset: DVec2::new(ob.x as f64, ob.y as f64),
            visible: ob.visible,
            parent_id,
            child_ids: vec![],
            object_ids: vec![],
            shape: Some(geometry::Shape::from(&ob.shape)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtractNode {
    pub id: ExtractNodeId,
    /// The custom class assigned to the extracted element in the source map.
    pub user_type: String,
    /// A copy of the extracted element's properties table.
    pub properties: tiled::Properties,
    /// Spatial offset relative to parent node.
    pub offset: DVec2,
    /// If the extracted element was set to be visible in the editor.
    pub visible: bool,
    /// The ID of this node's parent. Top-level layers will be given the special `Root`
    /// parent (the map itself). Everything else will have a layer as its parent.
    pub parent_id: ExtractNodeId,
    /// Child layers as extracted IDs. Only Group layers can have sublayers.
    pub child_ids: Vec<ExtractNodeId>,
    /// Objects contained by this node as extracted IDs.
    pub object_ids: Vec<ExtractNodeId>,
    /// Optional shape
    pub shape: Option<geometry::Shape>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtractNodeId {
    Root,
    Layer(u32),
    Object(u32),
}

impl ConvertProperty for ExtractNodeId {
    fn convert_property(propval: &tiled::PropertyValue) -> Result<Self, PropertyErrorKind> {
        match propval {
            tiled::PropertyValue::ObjectValue(o) => Ok(ExtractNodeId::Object(*o)),
            _ => Err(PropertyErrorKind::UnexpectedType),
        }
    }
}

/// Provides access to an extracted node within its extraction context.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ExtractNodeAccess<'root> {
    pub(crate) root: &'root Extract,
    pub(crate) data: &'root ExtractNode,
}

impl<'root> std::ops::Deref for ExtractNodeAccess<'root> {
    type Target = ExtractNode;

    fn deref(&self) -> &'root Self::Target {
        self.data
    }
}

impl<'root> ExtractNodeAccess<'root> {
    pub(crate) fn new(root: &'root Extract, data: &'root ExtractNode) -> Self {
        Self { root, data }
    }

    /// Get the context this node belongs to.
    pub fn root(&self) -> &'root Extract {
        self.root
    }

    /// Access the parent of this node, if it has one. Nodes extracted from top-level layers will
    /// not have a parent.
    pub fn parent(&self) -> Option<ExtractNodeAccess<'root>> {
        self.root().access(&self.parent_id)
    }

    /// Provides an iterator over the map objects extracted with this node.
    pub fn objects(&self) -> impl Iterator<Item = ExtractNodeAccess<'_>> {
        self.object_ids
            .iter()
            .filter_map(|id| self.root().access(id))
    }

    /// Calculates the global position of this node, taking its ancestors' pose/s into account.
    pub fn global_position(&self) -> DVec2 {
        if let Some(parent) = self.parent() {
            parent.global_position() + self.offset
        } else {
            self.offset
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct ExtractMapInfo {
    pub source: PathBuf,
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
}

impl ExtractMapInfo {
    pub fn from_map(tmx: &tiled::Map) -> Self {
        Self {
            source: tmx.source.clone(),
            width: tmx.width,
            height: tmx.height,
            tile_width: tmx.tile_width,
            tile_height: tmx.tile_height,
        }
    }
}
