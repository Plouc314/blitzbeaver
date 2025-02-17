use unicode_segmentation::UnicodeSegmentation;

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
