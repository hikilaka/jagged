use super::util;
use std::collections::HashMap;

// some possible issues w/ current design: excessive copying??
// a lot of the interfacing is a clear copy on HashMap.. maybe we can
// leverage that.

#[derive(Clone, Debug, Default)]
pub struct Archive {
    pub(crate) entries: HashMap<u32, Vec<u8>>,
}

impl Archive {
    /// Creates a new archive with an empty entry table.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Archive {
    /// Returns `true` if the entry table contains the specified key.
    pub fn contains_key<S>(&self, key: S) -> bool
    where
        S: Into<String>,
    {
        let hash = util::entry_hash(key.into());

        self.entries.contains_key(&hash)
    }

    /// Removes an entry.
    pub fn remove<S>(&mut self, key: S) -> Option<Vec<u8>>
    where
        S: Into<String>,
    {
        let hash = util::entry_hash(key.into());

        self.entries.remove(&hash)
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
}
