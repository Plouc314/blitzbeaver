use unicode_segmentation::UnicodeSegmentation;

/// Word
///
/// This is a string type that is optimized for distance calculations.
///
/// The graphemes store the same string as a sequence of grapheme clusters.
/// Each grapheme is stored as a u64, this is an optimization and is not always valid
/// (that is there exists grapheme that are larger than 8 bytes). However in practice
/// grapheme will (almost) always be smaller than 8 bytes.
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Word<'a> {
    pub raw: &'a str,
    pub graphemes: Vec<u64>,
}

impl<'a> Word<'a> {
    pub fn new(raw: &'a str) -> Self {
        Self {
            raw,
            graphemes: raw
                .graphemes(true)
                .map(|g| {
                    if g.len() > 8 {
                        println!("Grapheme cluster larger than 8 bytes: {}", g);
                        Self::pack_big_endian(&g.as_bytes()[..8])
                    } else {
                        Self::pack_big_endian(g.as_bytes())
                    }
                })
                .collect(),
        }
    }

    fn pack_big_endian(bytes: &[u8]) -> u64 {
        bytes
            .iter()
            .fold(0u64, |acc, &byte| (acc << 8) | byte as u64)
    }
}
