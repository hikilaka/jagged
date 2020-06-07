mod archive;
mod read;
mod util;
mod write;

pub use archive::*;
pub use read::*;
pub use write::*;

pub trait ReadWriteArchive: ReadArchive + WriteArchive {}
