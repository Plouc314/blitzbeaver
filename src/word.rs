use log::warn;
use unicode_segmentation::UnicodeSegmentation;

pub type GraphemeType = u64;

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
    pub graphemes: Vec<GraphemeType>,
}

impl Word {
    pub fn new(raw: String) -> Self {
        Self {
            graphemes: raw
                .as_str()
                .graphemes(true)
                .map(|g| Self::string_to_grapheme(g))
                .collect(),
            raw,
        }
    }

    pub fn from_graphemes(graphemes: Vec<GraphemeType>) -> Self {
        Self {
            raw: graphemes
                .iter()
                .map(|&g| Self::grapheme_to_string(g))
                .collect::<Vec<String>>()
                .join(""),
            graphemes,
        }
    }

    pub fn grapheme_to_string(grapheme: GraphemeType) -> String {
        String::from_utf8(Self::unpack_grapheme(grapheme))
            .unwrap()
            .to_string()
    }

    pub fn string_to_grapheme(s: &str) -> GraphemeType {
        if s.len() > 8 {
            warn!("Grapheme cluster larger than 8 bytes: {}", s);
            Self::pack_grapheme(&s.as_bytes()[..8])
        } else {
            Self::pack_grapheme(s.as_bytes())
        }
    }

    fn pack_grapheme(bytes: &[u8]) -> GraphemeType {
        bytes
            .iter()
            .fold(0u64, |acc, &byte| (acc << 8) | byte as u64)
    }

    fn unpack_grapheme(val: GraphemeType) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8);
        let mut val = val;
        for _ in 0..8 {
            let byte = (val & 0xFF) as u8;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
            val >>= 8;
        }
        bytes.reverse();
        bytes
    }
}
