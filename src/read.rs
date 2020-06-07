use super::{util, Archive};

use anyhow::{anyhow, Error, Result};
use bytes::{Buf, Bytes};
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub trait ReadArchive {
    /// Gets a reference to an entry from an archive, if it exists.
    fn get<S>(&self, key: S) -> Option<&Vec<u8>>
    where
        S: Into<String>;

    /// Gets a mutable reference to an entry from an archive, if it exists.
    fn get_mut<S>(&mut self, key: S) -> Option<&mut Vec<u8>>
    where
        S: Into<String>;
}

impl ReadArchive for Archive {
    fn get<S>(&self, key: S) -> Option<&Vec<u8>>
    where
        S: Into<String>,
    {
        let hash = util::entry_hash(key.into());

        self.entries.get(&hash)
    }

    fn get_mut<S>(&mut self, key: S) -> Option<&mut Vec<u8>>
    where
        S: Into<String>,
    {
        let hash = util::entry_hash(key.into());

        self.entries.get_mut(&hash)
    }
}

/// Implementation of reading the archive from disk.
impl Archive {
    fn read_archive(&mut self, buffer: Bytes) -> Result<()> {
        let buffer = match self.read_headers(buffer) {
            Ok(header) => header,
            Err(err) => return Err(err),
        };

        self.read_entries(buffer)?;

        Ok(())
    }

    fn read_headers(&mut self, mut buffer: Bytes) -> Result<Bytes> {
        if buffer.remaining() < 6 {
            return Err(anyhow!("Unexpected EOF: Unable to read archive headers"));
        }

        let decompressed_size = buffer.get_int(3);
        let compressed_size = buffer.get_int(3);

        if decompressed_size != compressed_size {
            // the archive requires decompressing, decompress it and make
            // sure there aren't any errors..
            match util::decompress(buffer) {
                Some(decompressed) => Ok(decompressed),
                None => Err(anyhow!(
                    "Invalid payload: Unable to decompress bzip2 stream"
                )),
            }
        } else {
            Ok(buffer)
        }
    }

    fn read_entries(&mut self, mut buffer: Bytes) -> Result<()> {
        if buffer.remaining() < 2 {
            return Err(anyhow!(
                "Unexpected EOF: Unable to read archive entry count"
            ));
        }

        let entry_count = buffer.get_u16();
        let data_start = (entry_count * 10) as usize;
        let mut data_buffer = buffer.split_off(data_start);

        for _ in 0..entry_count {
            if buffer.remaining() < 10 {
                return Err(anyhow!(
                    "Unexpected EOF: Unable to read archive entry count"
                ));
            }
            let hash = buffer.get_int(4) as u32;
            let decompressed_size = buffer.get_int(3) as usize;
            let compressed_size = buffer.get_int(3) as usize;

            let data = data_buffer.split_to(compressed_size);

            if decompressed_size != compressed_size {
                let data = match util::decompress(data) {
                    Some(decompressed) => Ok(decompressed),
                    None => Err(anyhow!(
                        "Invalid payload: Unable to decompress bzip2 stream"
                    )),
                }?;
                self.entries.insert(hash, data.to_vec());
            } else {
                self.entries.insert(hash, data.to_vec());
            }
        }

        Ok(())
    }
}

impl TryFrom<&Path> for Archive {
    type Error = Error;

    /// Attempts to read an existing archive from disk into a new archive.
    fn try_from(path: &Path) -> Result<Self, self::Error> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        // read the file to a vector
        file.read_to_end(&mut data)?;

        // then place that vector into a Bytes object
        let buffer = Bytes::from(data);

        let mut archive = Archive::new();

        // now we can attempt to read the archive's headers
        archive.read_archive(buffer)?;

        Ok(archive)
    }
}

#[cfg(test)]
mod read_tests {
    #[test]
    fn test_new_archive() {
        use super::*;

        let archive = Archive::new();

        assert_eq!(archive.len(), 0);
        assert_eq!(archive.contains_key("logo.tga"), false);
        assert_eq!(archive.get("whatever"), None);
    }

    #[test]
    fn test_existing_archive() {
        use super::*;

        let path = Path::new("./release/jagex.jag");

        // archive loaded without errors
        let archive = Archive::try_from(path);
        assert!(archive.is_ok());

        // check that jagex.jag archive contains 9 entries
        let mut archive = archive.unwrap();
        assert_eq!(archive.len(), 9);

        // check that reading entries works correctly
        assert_eq!(archive.contains_key("logo.tga"), true);

        let data = archive.get("logo.tga").unwrap();
        let mut tga_data = Vec::new();

        let mut tga_file = File::open("./release/logo.tga").unwrap();
        tga_file.read_to_end(&mut tga_data).unwrap();

        // make sure that what we read is correct..
        assert_eq!(*data, tga_data);

        // ensure that clear() works
        archive.clear();
        assert_eq!(archive.len(), 0);
    }
}
