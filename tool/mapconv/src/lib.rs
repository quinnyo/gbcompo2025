use std::io;

pub mod brush;
pub mod builder;
pub mod chunk;
pub mod elem;
pub mod geometry;
pub mod out;
pub mod placement;
pub mod tiled_ext;
pub mod tilemap;
pub mod tsr;

pub mod coord {
    pub use glam::{DVec2, I8Vec2, IVec2, U16Vec2, U8Vec2};
}

pub use tiled::Map as Tmx;
// pub use tsr::Tilesetter;

use builder::Builder;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConversionFailed,
    Custom(&'static str),
    Io(io::Error),
}

impl From<&'static str> for Error {
    fn from(v: &'static str) -> Self {
        Self::Custom(v)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

pub const PROP_EDITOR_ONLY: &str = "editor_only";

pub fn process_tmx(builder: &mut Builder, tmx: Tmx) {
    let map_name = tmx.source.file_stem().unwrap().to_str().unwrap();

    eprintln!("Building map '{map_name}' from tmx file {:?}", tmx.source);
    builder.name = map_name.to_string();

    for layer in tmx.layers() {
        if !tiled_ext::properties_get_bool(&layer.properties, PROP_EDITOR_ONLY).unwrap_or(false) {
            builder.extract_layer(&layer);
        }
    }
}
