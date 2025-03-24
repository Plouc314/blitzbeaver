use std::{cmp::max, usize};

use unicode_segmentation::UnicodeSegmentation;

use crate::word::{GraphemeType, Word};

use super::sigmoid;

#[derive(Debug, Clone)]
pub enum InternalDistanceMetricConfig {
    Lv(bool),
    LvOpti(bool),
    LvEdit(f32, f32, f32, bool),
    LvSubstring(f32, bool),
    LvMultiWord(GraphemeType, bool),
}

impl InternalDistanceMetricConfig {
    pub fn make_metric(&self) -> Box<dyn DistanceMetric<Word> + Send + Sync> {
        match self {
            InternalDistanceMetricConfig::Lv(use_sigmoid) => {
                Box::new(LvDistanceMetric::new(*use_sigmoid))
            }
            InternalDistanceMetricConfig::LvOpti(use_sigmoid) => {
                Box::new(LvOptiDistanceMetric::new(*use_sigmoid))
            }
            InternalDistanceMetricConfig::LvEdit(sub, del, add, use_sigmoid) => {
                Box::new(LvEditDistanceMetric::new(*sub, *del, *add, *use_sigmoid))
            }
            InternalDistanceMetricConfig::LvSubstring(weight, use_sigmoid) => {
                Box::new(LvSubstringDistanceMetric::new(*weight, *use_sigmoid))
            }
            InternalDistanceMetricConfig::LvMultiWord(separator, use_sigmoid) => {
                Box::new(LvMultiWordDistanceMetric::new(*separator, *use_sigmoid))
            }
        }
    }
}

/// DistanceMetric
///
/// Defines a metric to compute the distance between two elements.
/// The distance must be in the range `[0, 1]`, where 1 means the elements are equal
/// and 0 means they are completely different.
///
/// Note: this is defined as a trait instead of a simple function to allow for the use of buffers.
pub trait DistanceMetric<T: ?Sized> {
    fn dist(&mut self, v1: &T, v2: &T) -> f32;

    /// Clone the distance metric
    ///
    /// This is done this way because of restrictions on the trait
    /// due to it being used as dyn DistanceMetric.
    fn clone(&self) -> Box<dyn DistanceMetric<T> + Send + Sync>;
}

/// Levenshtein Distance Metric
///
/// This metric computes the Levenshtein distance between two words.
pub struct LvDistanceMetric {
    use_sigmoid: bool,
}

impl LvDistanceMetric {
    pub fn new(use_sigmoid: bool) -> Self {
        Self { use_sigmoid }
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
        let dist = 1.0 - edits as f32 / usize::max(v1.raw.len(), v2.raw.len()) as f32;
        if self.use_sigmoid {
            sigmoid(dist)
        } else {
            dist
        }
    }

    fn clone(&self) -> Box<dyn DistanceMetric<Word> + Send + Sync> {
        Box::new(LvDistanceMetric::new(self.use_sigmoid))
    }
}

/// Optimized Levenshtein Distance Metric
///
/// This metric computes the Levenshtein distance between two words.
pub struct LvOptiDistanceMetric {
    dp: Vec<u8>,
    use_sigmoid: bool,
}

impl LvOptiDistanceMetric {
    pub fn new(use_sigmoid: bool) -> Self {
        Self {
            use_sigmoid,
            dp: Vec::new(),
        }
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
        let dist = 1.0 - edits as f32 / usize::max(v1.raw.len(), v2.raw.len()) as f32;
        if self.use_sigmoid {
            sigmoid(dist)
        } else {
            dist
        }
    }

    fn clone(&self) -> Box<dyn DistanceMetric<Word> + Send + Sync> {
        Box::new(LvOptiDistanceMetric::new(self.use_sigmoid))
    }
}

/// Levenshtein Edit
///
/// Represents an edit operation in the Levenshtein distance computation.
#[derive(Debug, Clone)]
pub enum LvEdit {
    /// Substitution: replace the grapheme at index `usize` with the grapheme `GraphemeType`
    /// The index is the index of the grapheme in the target word.
    Sub(usize, GraphemeType),
    /// Deletion: delete the grapheme at index `usize`
    /// The index is the index of the grapheme in the source word.
    Del(usize),
    /// Addition: add the grapheme `GraphemeType` at index `usize`
    /// The index is the index of the grapheme in the target word.
    Add(usize, GraphemeType),
}

impl Default for LvEdit {
    fn default() -> Self {
        LvEdit::Sub(0, 0)
    }
}

/// Levenshtein Edit Node
///
/// Represents a node in the dynamic programming table used to compute the Levenshtein distance.
/// It contains the distance to the target word, the edit operation to reach this distance and the
/// index of the previous node in the table.
#[derive(Debug, Clone, Default)]
pub struct LvEditNode {
    pub dist: u8,
    pub edit: LvEdit,
    pub prev: usize,
}

impl LvEditNode {
    pub fn new(dist: u8, edit: LvEdit, prev: usize) -> Self {
        Self { dist, edit, prev }
    }
}

/// Levenshtein Edit Distance Metric
///
/// This metric computes the Levenshtein distance between two words and computes the list of edits
/// to transform the source word into the target word.
pub struct LvEditDistanceMetric {
    dp: Vec<LvEditNode>,
    sub_weight: f32,
    del_weight: f32,
    add_weight: f32,
    use_sigmoid: bool,
}

impl LvEditDistanceMetric {
    pub fn new(sub_weight: f32, del_weight: f32, add_weight: f32, use_sigmoid: bool) -> Self {
        Self {
            dp: Vec::new(),
            sub_weight,
            del_weight,
            add_weight,
            use_sigmoid,
        }
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
                self.dp.push(LvEditNode::default());
            }
        }

        // clear all elems
        self.dp.fill(LvEditNode::default());
    }

    /// Get the list of edits to transform the source word into the target word.
    ///
    /// The list of edits is computed by backtracking the dynamic programming table,
    /// it is returned in reverse order.
    fn get_edit_list(&self, idx: usize) -> Vec<LvEdit> {
        let mut edits = Vec::new();
        let mut idx = idx;
        loop {
            let node = &self.dp[idx];
            if node.dist == 0 {
                break;
            }
            edits.push(node.edit.clone());
            idx = node.prev;
        }
        edits.reverse();
        edits
    }

    /// Compute the list of edits to transform the source word into the target word.
    pub fn compute_edits<'a>(&mut self, src: &'a Word, trg: &'a Word) -> Vec<LvEdit> {
        let len_src = src.graphemes.len();
        let len_trg = trg.graphemes.len();

        self.setup_dp(len_src, len_trg);

        for i in 1..(len_src + 1) {
            let idx = Self::idx_at(i, 0, len_trg);
            self.dp[idx] =
                LvEditNode::new(i as u8, LvEdit::Del(i - 1), Self::idx_at(i - 1, 0, len_trg));
        }

        for j in 1..(len_trg + 1) {
            let idx = Self::idx_at(0, j, len_trg);
            self.dp[idx] = LvEditNode::new(
                j as u8,
                LvEdit::Add(j - 1, trg.graphemes[j - 1]),
                Self::idx_at(0, j - 1, len_trg),
            );
        }

        for i in 1..(len_src + 1) {
            let g1 = src.graphemes[i - 1];
            for j in 1..(len_trg + 1) {
                let g2 = trg.graphemes[j - 1];

                let idx_cur = Self::idx_at(i, j, len_trg);
                if g1 == g2 {
                    let idx_prev = Self::idx_at(i - 1, j - 1, len_trg);
                    self.dp[idx_cur] = self.dp[idx_prev].clone();
                    continue;
                }

                let idx_sub = Self::idx_at(i - 1, j - 1, len_trg);
                let idx_del = Self::idx_at(i - 1, j, len_trg);
                let idx_add = Self::idx_at(i, j - 1, len_trg);

                let node_sub = &self.dp[idx_sub];
                let node_del = &self.dp[idx_del];
                let node_add = &self.dp[idx_add];

                if node_sub.dist < node_del.dist && node_sub.dist < node_add.dist {
                    self.dp[idx_cur] = LvEditNode::new(
                        node_sub.dist + 1,
                        LvEdit::Sub(j - 1, trg.graphemes[j - 1]),
                        idx_sub,
                    );
                } else if node_del.dist < node_add.dist {
                    self.dp[idx_cur] =
                        LvEditNode::new(node_del.dist + 1, LvEdit::Del(i - 1), idx_del);
                } else {
                    self.dp[idx_cur] = LvEditNode::new(
                        node_add.dist + 1,
                        LvEdit::Add(j - 1, trg.graphemes[j - 1]),
                        idx_add,
                    );
                }
            }
        }

        let idx = Self::idx_at(len_src, len_trg, len_trg);
        self.get_edit_list(idx)
    }
}

fn get_edits_counts(edits: Vec<LvEdit>) -> (f32, f32, f32) {
    let mut sub_count = 0.;
    let mut del_count = 0.;
    let mut add_count = 0.;
    for edit in edits {
        match edit {
            LvEdit::Sub(_index, _grapheme) => {
                sub_count += 1.;
            }
            LvEdit::Del(_index) => {
                del_count += 1.;
            }
            LvEdit::Add(_index, _grapheme) => {
                add_count += 1.;
            }
        }
    }
    (sub_count, del_count, add_count)
}

impl DistanceMetric<Word> for LvEditDistanceMetric {
    fn dist(&mut self, v1: &Word, v2: &Word) -> f32 {
        let all_edits = self.compute_edits(v1, v2);
        let (sub_count, del_count, add_count) = get_edits_counts(all_edits);
        let edit_count =
            sub_count * self.sub_weight + del_count * self.del_weight + add_count * self.add_weight;
        let dist = 1.0 - edit_count / usize::max(v1.raw.len(), v2.raw.len()) as f32;
        if self.use_sigmoid {
            sigmoid(dist)
        } else {
            dist
        }
    }

    fn clone(&self) -> Box<dyn DistanceMetric<Word> + Send + Sync> {
        Box::new(LvEditDistanceMetric::new(
            self.sub_weight,
            self.del_weight,
            self.add_weight,
            self.use_sigmoid,
        ))
    }
}

pub struct LvSubstringDistanceMetric {
    dplv: Vec<u8>,
    dpss: Vec<u8>,
    weight: f32,
    use_sigmoid: bool,
}

impl LvSubstringDistanceMetric {
    pub fn new(weight: f32, use_sigmoid: bool) -> Self {
        Self {
            dplv: Vec::new(),
            dpss: Vec::new(),
            weight,
            use_sigmoid,
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

    fn compute_edits<'a>(&mut self, mut w1: &'a Word, mut w2: &'a Word) -> (u8, u8) {
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
            self.dplv[idx] = i as u8;
        }

        // set a boundary after the diagonal
        for j in 1..(len_w2 + 1) {
            let idx = Self::idx_at(j - 1, j, len_w2);
            self.dplv[idx] = 255;
        }

        let mut longest = 0;

        for i in 1..(len_w1 + 1) {
            let g1 = w1.graphemes[i - 1];

            let to = usize::min(i + 1, len_w2 + 1);
            for j in 1..(to) {
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
        let bonus = longest_substring as f32 * self.weight;
        let edits = f32::max(edits as f32 - bonus, 0.0);
        let dist = 1.0 - edits / usize::max(v1.raw.len(), v2.raw.len()) as f32;
        if self.use_sigmoid {
            sigmoid(dist)
        } else {
            dist
        }
    }

    fn clone(&self) -> Box<dyn DistanceMetric<Word> + Send + Sync> {
        Box::new(LvSubstringDistanceMetric::new(
            self.weight,
            self.use_sigmoid,
        ))
    }
}

pub struct LvMultiWordDistanceMetric {
    lvopti: LvOptiDistanceMetric,
    separator: GraphemeType,
    use_sigmoid: bool,
}

impl LvMultiWordDistanceMetric {
    pub fn new(separator: GraphemeType, use_sigmoid: bool) -> Self {
        Self {
            lvopti: LvOptiDistanceMetric::new(false),
            separator,
            use_sigmoid,
        }
    }

    fn extract_multiple_words(w: &Word, separator: GraphemeType) -> Vec<Vec<GraphemeType>> {
        let mut ret: Vec<Vec<GraphemeType>> = Vec::new();
        let mut temp: Vec<GraphemeType> = Vec::new();

        for g in &w.graphemes {
            if *g == separator {
                if !temp.is_empty() {
                    ret.push(temp);
                }
                temp = Vec::new();
            } else {
                temp.push(*g);
            }
        }
        if !temp.is_empty() {
            ret.push(temp);
        }
        ret
    }

    fn compute_edits(&mut self, w1: &Word, w2: &Word) -> u8 {
        let multi_w1 = Self::extract_multiple_words(w1, self.separator);
        let multi_w2 = Self::extract_multiple_words(w2, self.separator);

        let mut total = 0;
        let mut perfect_matches = 0;

        let len1 = multi_w1.len();
        let len2 = multi_w2.len();

        if !(len1 == len2 && len1 > 1) {
            return self.lvopti.compute_edits(w1, w2);
        }
        for i in 0..len1 {
            let w1 = &Word::from_graphemes(multi_w1[i].clone());
            let w2 = &Word::from_graphemes(multi_w2[i].clone());

            let edits = self.lvopti.compute_edits(w1, w2);
            if edits == 0 {
                perfect_matches += 1;
            }
            total += edits;
        }

        let factor = 1.0 - (perfect_matches as f32) / (len1 as f32);
        return (total as f32 * factor) as u8;
    }
}

impl DistanceMetric<Word> for LvMultiWordDistanceMetric {
    fn dist(&mut self, v1: &Word, v2: &Word) -> f32 {
        let edits = self.compute_edits(v1, v2);
        let dist = 1.0 - edits as f32 / usize::max(v1.raw.len(), v2.raw.len()) as f32;
        if self.use_sigmoid {
            sigmoid(dist)
        } else {
            dist
        }
    }

    fn clone(&self) -> Box<dyn DistanceMetric<Word> + Send + Sync> {
        Box::new(LvMultiWordDistanceMetric::new(
            self.separator,
            self.use_sigmoid,
        ))
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
    fn test_lv_distance_metric() {
        let mut metric = LvDistanceMetric::new(false);

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
    fn test_lv_opti_distance_metric() {
        let mut metric = LvOptiDistanceMetric::new(false);

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
    fn test_lv_substring_metric() {
        let mut metric = LvSubstringDistanceMetric::new(1.0, false);

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

    #[test]
    fn test_lv_multi_word_metric() {
        let separator = Word::string_to_grapheme(" ");
        let mut metric = LvMultiWordDistanceMetric::new(separator, false);
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

        let w1 = create_word("inspecteur des impots");
        let w2 = create_word("insp des impots");
        let distance = metric.compute_edits(&w1, &w2);
        // 2 perfect matches => 6 * (1 - 2/3) == 1 (not 2)
        assert_eq!(distance, 1);

        let w1 = create_word("inspecteur impots");
        let w2 = create_word("insp des impots");
        let distance = metric.compute_edits(&w1, &w2);
        // classic distance
        assert_eq!(distance, 5);

        let w1 = create_word("inspecteur des impots");
        let w2 = create_word("inspe ds impts");
        let distance = metric.compute_edits(&w1, &w2);
        assert_eq!(distance, 7);
    }
}
