use mapconv::{MapConverter, Rgbasm};
use std::path::Path;
use std::{env, io};
use tiled::Loader;

#[derive(Debug)]
pub enum Error {
    ParameterMissing,
    FileNotFound,
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(Error::ParameterMissing);
    }
    let path = Path::new(&args[1]);
    if !path.is_file() {
        return Err(Error::FileNotFound);
    }
    let mut loader = Loader::new();
    let tmx = loader.load_tmx_map(path).unwrap();

    let mut converter = MapConverter::default();
    converter.process_tmx(tmx)?;
    converter.rgbasm(std::io::stdout())?;
    Ok(())
}
