use unicode_segmentation::UnicodeSegmentation;

/// Word
///
/// This is a string type that is optimized for distance calculations.
///
/// The raw string is owned and not a reference to the original string because of
/// complications with lifetime management. Also the array of clusters takes more
/// memory than the raw string anyway.
///
/// The graphemes store the same string as a sequence of grapheme clusters.
/// Each grapheme is stored as a u64, this is an optimization and is not always valid
/// (that is there exists grapheme that are larger than 8 bytes). However in practice
/// grapheme will (almost) always be smaller than 8 bytes.
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Word {
    pub raw: String,
    pub graphemes: Vec<u64>,
}

impl Word {
    pub fn new(raw: String) -> Self {
        Self {
            graphemes: raw
                .as_str()
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
            raw,
        }
    }

    fn pack_big_endian(bytes: &[u8]) -> u64 {
        bytes
            .iter()
            .fold(0u64, |acc, &byte| (acc << 8) | byte as u64)
    }
}
