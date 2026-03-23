use std::io;

pub mod brush;
pub mod chunk;
pub mod convert;
pub mod elem;
pub mod extract;
pub mod flow;
pub mod geometry;
pub mod out;
pub mod tiled_ext;
pub mod tilemap;
pub mod tsr;

pub mod coord {
    pub use glam::{DVec2, I8Vec2, IVec2, U16Vec2, U8Vec2};
}

pub use tiled::Map as Tmx;

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

pub fn process_tmx(tmx: Tmx) -> out::Map {
    let map_name = tmx.source.file_stem().unwrap().to_str().unwrap();
    eprintln!("Building map '{map_name}' from tmx file {:?}", tmx.source);

    let extract = extract::Extract::extract_tmx(&tmx);
    // for (id, node) in &extract.nodes {
    //     eprintln!("{id:?}: {node:?}");
    // }
    let mut conversion = convert::Conversion::from_extract(extract);
    convert::these_converters::submit(&mut conversion);
    let converted = conversion.convert();
    out::Map::new(map_name.to_string(), converted.resources, converted.chunks)
}
