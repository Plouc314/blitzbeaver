use std::usize;

use unicode_segmentation::UnicodeSegmentation;

use crate::word::Word;

pub trait Distance<T: ?Sized> {
    fn dist(&mut self, v1: &T, v2: &T) -> f32;
}

pub struct DummyDistance;

impl Distance<str> for DummyDistance {
    fn dist(&mut self, _v1: &str, _v2: &str) -> f32 {
        0.9
    }
}

pub struct LvDistance {}

impl LvDistance {
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

impl Distance<Word<'_>> for LvDistance {
    fn dist(&mut self, v1: &Word, v2: &Word) -> f32 {
        let edits = self.compute_edits(v1, v2);
        1.0 - edits as f32 / usize::max(v1.raw.len(), v2.raw.len()) as f32
    }
}

pub struct LvOptiDistance {
    dp: Vec<u8>,
}

impl LvOptiDistance {
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

    fn compute_edits<'a>(&mut self, mut w1: &'a Word<'a>, mut w2: &'a Word<'a>) -> u8 {
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

impl Distance<Word<'_>> for LvOptiDistance {
    fn dist(&mut self, v1: &Word, v2: &Word) -> f32 {
        let edits = self.compute_edits(v1, v2);
        1.0 - edits as f32 / usize::max(v1.raw.len(), v2.raw.len()) as f32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edit {
    insertion: u8,
    deletion: u8,
    substitution: u8,
}

impl Edit {
    pub fn new() -> Self {
        Self {
            insertion: 0,
            deletion: 0,
            substitution: 0,
        }
    }

    pub fn tot(&self) -> u8 {
        self.insertion + self.deletion + self.substitution
    }
}

pub struct LvMultiDistance {}

impl LvMultiDistance {
    pub fn new() -> Self {
        Self {}
    }

    fn idx_at(i: usize, j: usize, len_w2: usize) -> usize {
        i * (len_w2 + 1) + j
    }

    fn compute_edits(&mut self, w1: &str, w2: &str) -> u8 {
        let graphemes1 = w1.graphemes(true).collect::<Vec<&str>>();
        let graphemes2 = w2.graphemes(true).collect::<Vec<&str>>();

        let len_w1 = graphemes1.len();
        let len_w2 = graphemes2.len();

        let size = (len_w1 + 1) * (len_w2 + 1);
        let mut dp = Vec::with_capacity(size);
        for _ in 0..size {
            dp.push(Edit::new());
        }

        for i in 1..(len_w1 + 1) {
            for _ in 0..i {
                let idx = Self::idx_at(i, 0, len_w2);
                dp[idx].deletion += 1;
            }
        }

        for j in 1..(len_w2 + 1) {
            for _ in 0..j {
                let idx = Self::idx_at(0, j, len_w2);
                dp[idx].deletion += 1;
            }
        }

        for i in 1..(len_w1 + 1) {
            let g1 = graphemes1[i - 1];
            for j in 1..(len_w2 + 1) {
                let g2 = graphemes2[j - 1];

                let idx_cur = Self::idx_at(i, j, len_w2);
                if g1 == g2 {
                    let idx_prev = Self::idx_at(i - 1, j - 1, len_w2);
                    dp[idx_cur] = dp[idx_prev].clone();
                    continue;
                }

                let idx_sub = Self::idx_at(i - 1, j - 1, len_w2);
                let idx_del = Self::idx_at(i - 1, j, len_w2);
                let idx_add = Self::idx_at(i, j - 1, len_w2);

                let len_sub = dp[idx_sub].tot();
                let len_del = dp[idx_del].tot();
                let len_add = dp[idx_add].tot();

                let min_len = len_sub.min(len_del.min(len_add));

                if min_len == len_sub {
                    dp[idx_cur] = dp[idx_sub];
                    dp[idx_cur].substitution += 1;
                } else if min_len == len_del {
                    dp[idx_cur] = dp[idx_del];
                    dp[idx_cur].deletion += 1;
                } else {
                    dp[idx_cur] = dp[idx_add];
                    dp[idx_cur].insertion += 1;
                }
            }
        }

        let idx = Self::idx_at(len_w1, len_w2, len_w2);
        dp[idx].tot()
    }
}

impl Distance<str> for LvMultiDistance {
    fn dist(&mut self, v1: &str, v2: &str) -> f32 {
        let n_edits = self.compute_edits(v1, v2);
        1.0 - n_edits as f32 / usize::max(v1.len(), v2.len()) as f32
    }
}
