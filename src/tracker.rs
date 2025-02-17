use crate::{distances::CachedDistanceCalculatorWord, frame::Frame, word::Word};

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct RecordScore {
    idx: usize,
    score: f32,
}

impl RecordScore {
    pub fn new(idx: usize, score: f32) -> Self {
        Self { score, idx }
    }
}

/// For RecordScore reflexivity holds because score is never NaN
impl Eq for RecordScore {}

impl Ord for RecordScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.total_cmp(&other.score)
    }
}

pub struct TrackingNode<'a> {
    frame_idx: usize,
    record: Vec<Word<'a>>,
}

impl<'a> TrackingNode<'a> {
    pub fn new(frame_idx: usize, record: Vec<Word<'a>>) -> Self {
        Self { frame_idx, record }
    }
}

pub struct TrackerConfig {
    interest_threshold: f32,
}

pub struct Tracker<'a> {
    config: TrackerConfig,
    /// Sorted on the nodes frame_idx
    nodes: Vec<TrackingNode<'a>>,
}

impl<'a> Tracker<'a> {
    pub fn new(config: TrackerConfig) -> Self {
        Self {
            config,
            nodes: Vec::new(),
        }
    }

    pub fn add_tracking_node(&mut self, node: TrackingNode<'a>) {
        for i in 0..self.nodes.len() {
            if self.nodes[i].frame_idx > node.frame_idx {
                self.nodes.insert(i, node);
                return;
            }
        }
        self.nodes.push(node);
    }

    pub fn compute_score_record(
        &self,
        idx_record: usize,
        frame: &Frame<'a>,
        cached_distances: &mut Vec<CachedDistanceCalculatorWord<'a>>,
    ) -> f32 {
        let mut tot_score = 0.0;
        let node = self.nodes.last().unwrap();
        for idx_feature in 0..frame.columns().len() {
            let cached_distance = &mut cached_distances[idx_feature];
            let current_word = &node.record[idx_feature];
            let word = &frame.columns()[idx_feature][idx_record];
            let score = match word {
                Some(word) => cached_distance.get_dist(current_word, word),
                None => 0.0,
            };
            tot_score += score;
        }
        tot_score / frame.columns().len() as f32
    }

    pub fn scan_frame(
        &self,
        frame: &Frame<'a>,
        cached_distances: &mut Vec<CachedDistanceCalculatorWord<'a>>,
    ) -> Vec<RecordScore> {
        let mut scores = Vec::new();

        for idx in 0..frame.nrecs() {
            let score = self.compute_score_record(idx, frame, cached_distances);
            if score > self.config.interest_threshold {
                scores.push(RecordScore::new(idx, score));
            }
        }
        scores.sort_unstable();
        scores
    }
}
