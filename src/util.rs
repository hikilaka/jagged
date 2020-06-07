use bytes::Bytes;
use bzip2::read::BzDecoder;
use std::io::prelude::*;

pub(crate) fn decompress(data: Bytes) -> Option<Bytes> {
    // The required header, "BZh1", that is missing in jag archives
    // must be appended
    let mut concatenated = vec![66u8, 90, 104, 49];
    concatenated.extend(data.into_iter());

    let mut decompressor = BzDecoder::new(concatenated.as_slice());
    let mut decompressed_data = Vec::new();

    // we don't need to know the specific error, just that decompression
    // failed, hence why this func returns an Option rather than Result.
    match decompressor.read_to_end(&mut decompressed_data) {
        Ok(_) => Some(Bytes::from(decompressed_data)),
        Err(_) => None,
    }
}

pub(crate) fn entry_hash(entry: String) -> u32 {
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
mod util_tests {
    #[test]
    fn test_known_entry_hashes() {
        use std::collections::HashMap;

        let mut map = HashMap::new();

        map.insert("jagex.jag", 1827744299);
        map.insert("testing", 903262442);
        map.insert("hello.world", 241581634);
        map.insert("1234567890", 2318125913);
        map.insert("AbCdEfGhIJkLmNoPqRsTuVwXyZ", 4115466251);

        for (k, v) in map.into_iter() {
            assert_eq!(super::entry_hash(k.into()), v);
        }
    }
}
