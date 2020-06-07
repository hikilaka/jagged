use super::{util, Archive};

use anyhow::Result;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub trait WriteArchive {
    /// Writes an archive to disk with the given path. If a file is already
    /// present at the given path, it is simply overwritten.
    fn save(&self, path: &Path) -> Result<()>;

    /// Inserts a key-value pair. If the archive already contains the given
    /// key, the previous value for that key is returned.
    fn insert<S>(&mut self, key: S, value: Vec<u8>) -> Option<Vec<u8>>
    where
        S: Into<String>;
}

impl WriteArchive for Archive {
    fn save(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path)?;

        // compute the file's format and write to disk.
        self.write_to_file(&mut file)?;

        Ok(())
    }

    fn insert<S>(&mut self, key: S, value: Vec<u8>) -> Option<Vec<u8>>
    where
        S: Into<String>,
    {
        let hash = util::entry_hash(key.into());

        self.entries.insert(hash, value)
    }
}

impl Archive {
    fn write_to_file(&self, file: &mut File) -> Result<()> {
        Ok(())
    }

    fn generate_data_block(&self) -> Option<Vec<u8>> {
        None
    }
}
