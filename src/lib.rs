use bytes::{Buf, Bytes};
use bzip2::read::BzDecoder;
use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, Error, ErrorKind, Result};
use std::path::Path;

pub struct Archive {
    entries: HashMap<u32, Vec<u8>>,
}

impl Archive {
    /// Creates a new archive with an empty entry table. From here, you can
    /// call `load` to load an existing archive from disk.
    pub fn new() -> Archive {
        Archive {
            entries: HashMap::new(),
        }
    }

    /// Reads an existing archive from disk into this archive's entry table.
    /// If this archive already has entries from a previous call to `load`,
    /// the contents will simply be merged. Collisions in file hashes will be
    /// overwritten.
    pub fn load(&mut self, file_path: &Path) -> Result<()> {
        let mut file = File::open(file_path)?;
        let mut data = Vec::new();

        // read the file to a vector
        file.read_to_end(&mut data)?;

        // then place that vector into a Bytes object
        let buffer = Bytes::from(data);

        // now we can attempt to read the archive's headers
        self.read_archive(buffer)?;

        Ok(())
    }

    /// Gets a reference to an entry from an archive, if it exists.
    pub fn get<S>(&self, key: S) -> Option<&Vec<u8>>
    where
        S: Into<String>,
    {
        let hash = entry_hash(key.into());

        self.entries.get(&hash)
    }

    /// Gets a mutable reference to an entry from an archive, if it exists.
    pub fn get_mut<S>(&mut self, key: S) -> Option<&mut Vec<u8>>
    where
        S: Into<String>,
    {
        let hash = entry_hash(key.into());

        self.entries.get_mut(&hash)
    }

    /// Returns `true` if the entry table contains the specified key.
    pub fn contains_key<S>(&self, key: S) -> bool
    where
        S: Into<String>,
    {
        let hash = entry_hash(key.into());

        self.entries.contains_key(&hash)
    }

    /// Clears the entry table of this archive
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns the number of entries this archive contains.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the archive contains no entries
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn read_archive(&mut self, buffer: Bytes) -> Result<()> {
        let buffer = match self.read_headers(buffer) {
            Ok(header) => header,
            Err(err) => return Err(err),
        };

        self.read_entries(buffer)?;

        Ok(())
    }

    fn read_entries(&mut self, mut buffer: Bytes) -> Result<()> {
        if buffer.remaining() < 2 {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "Unable to read archive entry count",
            ));
        }

        let entry_count = buffer.get_u16();
        let data_start = (entry_count * 10) as usize;
        let mut data_buffer = buffer.split_off(data_start);

        for _ in 0..entry_count {
            if buffer.remaining() < 10 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "Unable to read archive entry",
                ));
            }
            let hash = buffer.get_int(4) as u32;
            let decompressed_size = buffer.get_int(3) as usize;
            let compressed_size = buffer.get_int(3) as usize;

            let data = data_buffer.split_to(compressed_size);

            if decompressed_size != compressed_size {
                let data = match decompress(data) {
                    Some(decompressed) => Ok(decompressed),
                    None => Err(Error::new(
                        ErrorKind::InvalidData,
                        "Unable to decompress Bzip2 stream",
                    )),
                }?;
                self.entries.insert(hash, data.to_vec());
            } else {
                self.entries.insert(hash, data.to_vec());
            }
        }

        Ok(())
    }

    fn read_headers(&mut self, mut buffer: Bytes) -> Result<Bytes> {
        if buffer.remaining() < 6 {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "Unable to read archive headers",
            ));
        }

        let decompressed_size = buffer.get_int(3);
        let compressed_size = buffer.get_int(3);

        if decompressed_size != compressed_size {
            // the archive requires decompressing, decompress it and make
            // sure there aren't any errors..
            match decompress(buffer) {
                Some(decompressed) => Ok(decompressed),
                None => Err(Error::new(
                    ErrorKind::InvalidData,
                    "Unable to decompress Bzip2 stream",
                )),
            }
        } else {
            Ok(buffer)
        }
    }
}

fn decompress(data: Bytes) -> Option<Bytes> {
    // The required header, "BZh1", that is missing in jag archives
    // must be appended
    let mut concatenated = vec![66u8, 90, 104, 49];
    concatenated.extend(data.into_iter());

    let mut decompressor = BzDecoder::new(concatenated.as_slice());
    let mut decompressed_data = Vec::new();

    match decompressor.read_to_end(&mut decompressed_data) {
        Ok(_) => Some(Bytes::from(decompressed_data)),
        Err(_) => None,
    }
}

fn entry_hash(entry: String) -> u32 {
    use std::num::Wrapping;

    let mut hash = Wrapping(0);
    let multiplier = Wrapping(61);

    for ch in entry.to_ascii_uppercase().as_bytes().iter() {
        hash *= multiplier;
        hash += Wrapping(*ch as u32 - 32);
    }

    hash.0
}

#[cfg(test)]
mod jagged_tests {
    #[test]
    fn test_known_entry_hashes() {
        use super::*;
        use std::collections::HashMap;

        let mut map = HashMap::new();

        map.insert("jagex.jag", 1827744299);
        map.insert("testing", 903262442);
        map.insert("hello.world", 241581634);
        map.insert("1234567890", 2318125913);
        map.insert("AbCdEfGhIJkLmNoPqRsTuVwXyZ", 4115466251);

        for (k, v) in map.into_iter() {
            assert_eq!(entry_hash(k.into()), v);
        }
    }

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

        let mut archive = Archive::new();
        let path = Path::new("./release/jagex.jag");

        // archive loaded without errors
        assert!(archive.load(&path).is_ok());

        // the jagex.jag archive contains 9 entries
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
