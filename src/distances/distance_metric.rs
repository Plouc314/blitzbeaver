use std::{
    cmp::{max, min},
    usize,
};

use unicode_segmentation::UnicodeSegmentation;

use crate::word::Word;

/// DistanceMetric
///
/// Defines a metric to compute the distance between two elements.
/// The distance must be in the range `[0, 1]`, where 1 means the elements are equal
/// and 0 means they are completely different.
///
/// Note: this is defined as a trait instead of a simple function to allow for the use of buffers.
pub trait DistanceMetric<T: ?Sized> {
    fn dist(&mut self, v1: &T, v2: &T) -> f32;
}

pub struct DummyDistanceMetric;

impl DistanceMetric<str> for DummyDistanceMetric {
    fn dist(&mut self, _v1: &str, _v2: &str) -> f32 {
        0.9
    }
}

/// Levenshtein Distance Metric
///
/// This metric computes the Levenshtein distance between two words.
pub struct LvDistanceMetric {}

impl LvDistanceMetric {
    pub fn new() -> Self {
        Self {}
    }

    fn idx_at(i: usize, j: usize, len_w2: usize) -> usize {
        i * (len_w2 + 1) + j
    }

    fn compute_edits(&mut self, w1: &Word, w2: &Word) -> u32 {
        // Backward compatibility
        let graphemes1 = w1.raw.graphemes(true).collect::<Vec<&str>>();
        let graphemes2 = w2.raw.graphemes(true).collect::<Vec<&str>>();

        let len_w1 = graphemes1.len();
        let len_w2 = graphemes2.len();

        let size = (len_w1 + 1) * (len_w2 + 1);
        let mut dp = Vec::with_capacity(size);
        for _ in 0..size {
            dp.push(0);
        }

        for i in 1..(len_w1 + 1) {
            let idx = Self::idx_at(i, 0, len_w2);
            dp[idx] = i as u32;
        }

        for j in 1..(len_w2 + 1) {
            let idx = Self::idx_at(0, j, len_w2);
            dp[idx] = j as u32;
        }

        for i in 1..(len_w1 + 1) {
            let g1 = graphemes1[i - 1];
            for j in 1..(len_w2 + 1) {
                let g2 = graphemes2[j - 1];

                let idx_cur = Self::idx_at(i, j, len_w2);
                if g1 == g2 {
                    let idx_prev = Self::idx_at(i - 1, j - 1, len_w2);
                    dp[idx_cur] = dp[idx_prev];
                    continue;
                }

                let idx_sub = Self::idx_at(i - 1, j - 1, len_w2);
                let idx_del = Self::idx_at(i - 1, j, len_w2);
                let idx_add = Self::idx_at(i, j - 1, len_w2);

                let len_sub = dp[idx_sub];
                let len_del = dp[idx_del];
                let len_add = dp[idx_add];

                let min_len = len_sub.min(len_del.min(len_add));

                if min_len == len_sub {
                    dp[idx_cur] = len_sub + 1;
                } else if min_len == len_del {
                    dp[idx_cur] = len_del + 1;
                } else {
                    dp[idx_cur] = len_add + 1;
                }
            }
        }

        let idx = Self::idx_at(len_w1, len_w2, len_w2);
        dp[idx]
    }
}

impl DistanceMetric<Word> for LvDistanceMetric {
    fn dist(&mut self, v1: &Word, v2: &Word) -> f32 {
        let edits = self.compute_edits(v1, v2);
        1.0 - edits as f32 / usize::max(v1.raw.len(), v2.raw.len()) as f32
    }
}

/// Optimized Levenshtein Distance Metric
///
/// This metric computes the Levenshtein distance between two words.
pub struct LvOptiDistanceMetric {
    dp: Vec<u8>,
}

impl LvOptiDistanceMetric {
    pub fn new() -> Self {
        Self { dp: Vec::new() }
    }

    fn idx_at(i: usize, j: usize, len_w2: usize) -> usize {
        i * (len_w2 + 1) + j
    }

    fn setup_dp(&mut self, len_w1: usize, len_w2: usize) {
        let size = (len_w1 + 1) * (len_w2 + 1);
        if size > self.dp.len() {
            let additional = size - self.dp.len();
            self.dp.reserve(additional);

            // create new elems
            for _ in 0..additional {
                self.dp.push(0);
            }
        }

        // clear all elems
        self.dp.fill(0);
    }

    fn compute_edits<'a>(&mut self, mut w1: &'a Word, mut w2: &'a Word) -> u8 {
        let mut len_w1 = w1.graphemes.len();
        let mut len_w2 = w2.graphemes.len();

        // w1 must be the largest word
        if len_w1 < len_w2 {
            (len_w1, len_w2) = (len_w2, len_w1);
            (w1, w2) = (w2, w1);
        }

        self.setup_dp(len_w1, len_w2);

        for i in 1..(len_w1 + 1) {
            let idx = Self::idx_at(i, 0, len_w2);
            self.dp[idx] = i as u8;
        }

        // set a boundary after the diagonal
        for j in 1..(len_w2 + 1) {
            let idx = Self::idx_at(j - 1, j, len_w2);
            self.dp[idx] = 255;
        }

        for i in 1..(len_w1 + 1) {
            let g1 = w1.graphemes[i - 1];

            // only iter to the diagonal
            let to = usize::min(i + 1, len_w2 + 1);
            for j in 1..(to) {
                let g2 = w2.graphemes[j - 1];

                let idx_cur = Self::idx_at(i, j, len_w2);
                if g1 == g2 {
                    let idx_prev = Self::idx_at(i - 1, j - 1, len_w2);
                    self.dp[idx_cur] = self.dp[idx_prev];
                    continue;
                }

                let idx_sub = Self::idx_at(i - 1, j - 1, len_w2);
                let idx_del = Self::idx_at(i - 1, j, len_w2);
                let idx_add = Self::idx_at(i, j - 1, len_w2);

                let len_sub = self.dp[idx_sub];
                let len_del = self.dp[idx_del];
                let len_add = self.dp[idx_add];

                if len_sub < len_del && len_sub < len_add {
                    self.dp[idx_cur] = len_sub + 1;
                } else if len_del < len_add {
                    self.dp[idx_cur] = len_del + 1;
                } else {
                    self.dp[idx_cur] = len_add + 1;
                }
            }
        }

        let idx = Self::idx_at(len_w1, len_w2, len_w2);
        self.dp[idx]
    }
}

impl DistanceMetric<Word> for LvOptiDistanceMetric {
    fn dist(&mut self, v1: &Word, v2: &Word) -> f32 {
        let edits = self.compute_edits(v1, v2);
        1.0 - edits as f32 / usize::max(v1.raw.len(), v2.raw.len()) as f32
    }
}

pub struct LvSubstringDistanceMetric {
    dplv: Vec<u8>,
    dpss: Vec<u8>,
    weight: f32,
}

impl LvSubstringDistanceMetric {
    pub fn new(weight: f32) -> Self {
        Self {
            dplv: Vec::new(),
            dpss: Vec::new(),
            weight: weight,
        }
    }

    fn idx_at(i: usize, j: usize, len_w2: usize) -> usize {
        i * (len_w2 + 1) + j
    }

    fn setup_dp(&mut self, len_w1: usize, len_w2: usize) {
        let size = (len_w1 + 1) * (len_w2 + 1);
        if size > self.dplv.len() {
            let additional = size - self.dplv.len();
            self.dplv.reserve(additional);
            self.dpss.reserve(additional);
            // create new elems
            for _ in 0..additional {
                self.dpss.push(0);
                self.dplv.push(0);
            }
        }

        // clear all elems
        self.dplv.fill(0);
        self.dpss.fill(0);
    }

    fn compute_edits(&mut self, w1: &Word, w2: &Word) -> (u8, u8) {
        let graphemes1 = w1.raw.graphemes(true).collect::<Vec<&str>>();
        let graphemes2 = w2.raw.graphemes(true).collect::<Vec<&str>>();

        let len_w1 = graphemes1.len();
        let len_w2 = graphemes2.len();

        self.setup_dp(len_w1, len_w2);

        for i in 1..(len_w1 + 1) {
            let idx = Self::idx_at(i, 0, len_w2);
            self.dplv[idx] = i as u8;
        }

        for j in 1..(len_w2 + 1) {
            let idx = Self::idx_at(0, j, len_w2);
            self.dplv[idx] = j as u8;
        }

        let mut longest = 0;

        for i in 1..(len_w1 + 1) {
            let g1 = w1.graphemes[i - 1];

            for j in 1..(len_w2 + 1) {
                let g2 = w2.graphemes[j - 1];

                let idx_cur = Self::idx_at(i, j, len_w2);
                if g1 == g2 {
                    let idx_prev = Self::idx_at(i - 1, j - 1, len_w2);
                    self.dplv[idx_cur] = self.dplv[idx_prev];
                    self.dpss[idx_cur] = self.dpss[idx_prev] + 1;
                    longest = max(longest, self.dpss[idx_cur]);
                    continue;
                }

                let idx_sub = Self::idx_at(i - 1, j - 1, len_w2);
                let idx_del = Self::idx_at(i - 1, j, len_w2);
                let idx_add = Self::idx_at(i, j - 1, len_w2);

                let len_sub = self.dplv[idx_sub];
                let len_del = self.dplv[idx_del];
                let len_add = self.dplv[idx_add];

                if len_sub < len_del && len_sub < len_add {
                    self.dplv[idx_cur] = len_sub + 1;
                } else if len_del < len_add {
                    self.dplv[idx_cur] = len_del + 1;
                } else {
                    self.dplv[idx_cur] = len_add + 1;
                }
            }
        }

        let idx = Self::idx_at(len_w1, len_w2, len_w2);
        (self.dplv[idx], longest)
    }
}

impl DistanceMetric<Word> for LvSubstringDistanceMetric {
    fn dist(&mut self, v1: &Word, v2: &Word) -> f32 {
        let (edits, longest_substring) = self.compute_edits(v1, v2);
        let bonus = (longest_substring as f32 * self.weight) as u8;
        let edits = max(edits - bonus, 0);
        1.0 - edits as f32 / usize::max(v1.raw.len(), v2.raw.len()) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a Word instance from a string
    fn create_word(s: &str) -> Word {
        Word::new(s.to_string())
    }

    #[test]
    fn test_lv_distance_metric_metric() {
        let mut metric = LvDistanceMetric::new();

        // Test identical words
        let w1 = create_word("hello");
        let w2 = create_word("hello");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 0);
        // Test one empty word
        let w2 = create_word("");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 5);

        // Test single character difference
        let w2 = create_word("hallo");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 1);

        // Test different lengths
        let w2 = create_word("helloworld");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 5); // More edits, distance should be lower

        // Test partial overlap
        let w1 = create_word("bernart");
        let w2 = create_word("jeanbernard");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 5); // Partial overlap, distance should be high (close to 1)

        // Test case sensitivity
        let w1 = create_word("Hello");
        let w2 = create_word("hello");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 1); // Case-sensitive comparison

        let w1 = create_word("Bernard");
        let w2 = create_word("bBeernard");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 2);
    }

    #[test]
    fn test_lv_opti_distance_metric_metric() {
        let mut metric = LvOptiDistanceMetric::new();

        // Test identical words
        let w1 = create_word("hello");
        let w2 = create_word("hello");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 0);
        // Test one empty word
        let w2 = create_word("");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 5);

        // Test single character difference
        let w2 = create_word("hallo");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 1);

        // Test different lengths
        let w2 = create_word("helloworld");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 5);

        // Test partial overlap
        let w1 = create_word("bernart");
        let w2 = create_word("jeanbernard");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 5);

        // Test case sensitivity
        let w1 = create_word("Hello");
        let w2 = create_word("hello");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 1); // Case-sensitive comparison

        let w1 = create_word("Bernard");
        let w2 = create_word("bBeernard");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 2);
    }

    #[test]
    fn test_lv_substring_metric_metric() {
        let mut metric = LvSubstringDistanceMetric::new(1.0);

        // Test identical words
        let w1 = create_word("hello");
        let w2 = create_word("hello");
        let (distance, longest) = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 0);
        assert_eq!(longest, 5);

        // Test one empty word
        let w2 = create_word("");
        let (distance, longest) = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 5);
        assert_eq!(longest, 0);

        // Test single character difference
        let w2 = create_word("hallo");
        let (distance, longest) = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 1);
        assert_eq!(longest, 3);

        // Test different lengths
        let w2 = create_word("helloworld");
        let (distance, longest) = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 5);
        assert_eq!(longest, 5);

        // Test partial overlap
        let w1 = create_word("bernart");
        let w2 = create_word("jeanbernard");
        let (distance, longest) = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 5);
        assert_eq!(longest, 6);

        // Test case sensitivity
        let w1 = create_word("Hello");
        let w2 = create_word("hello");
        let (distance, longest) = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 1); // Case-sensitive comparison
        assert_eq!(longest, 4);

        let w1 = create_word("Bernard");
        let w2 = create_word("bBeernard");
        let (distance, longest) = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 2);
        assert_eq!(longest, 6);
    }
}
