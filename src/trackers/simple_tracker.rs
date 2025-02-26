use crate::{
    api::{ChainNode, SimpleTrackerConfig},
    distances::CachedDistanceCalculator,
    frame::{Frame, Record},
    id::{self, ID},
};

use super::{
    tracker::{RecordScore, Tracker},
    TrackingChain,
};

#[derive(Clone)]
pub struct SimpleTracker {
    id: ID,
    config: SimpleTrackerConfig,
    chain: Vec<ChainNode>,
    record: Record,
}

impl SimpleTracker {
    pub fn new(config: SimpleTrackerConfig) -> Self {
        Self {
            id: id::new_id(),
            config,
            chain: Vec::new(),
            record: Record::default(),
        }
    }

    pub fn compute_distances(
        &self,
        frame: &Frame,
        distance_calculators: &mut Vec<CachedDistanceCalculator>,
    ) -> Vec<Vec<f32>> {
        let mut distances = Vec::new();

        for feature_idx in 0..frame.num_features() {
            let mut feature_distances = Vec::with_capacity(frame.num_records());
            let distance_calculator = &mut distance_calculators[feature_idx];
            let own_element = self.record.element(feature_idx);

            for element in frame.column(feature_idx).iter() {
                let dist = distance_calculator.get_dist(own_element, element);
                feature_distances.push(dist);
            }

            distances.push(feature_distances);
        }

        distances
    }

    pub fn compute_score_record(
        &self,
        frame: &Frame,
        distances: &Vec<Vec<f32>>,
        record_idx: usize,
    ) -> f32 {
        let mut tot = 0.0;
        for feature_idx in 0..frame.num_features() {
            tot += distances[feature_idx][record_idx];
        }
        tot / frame.num_features() as f32
    }
}

impl Tracker for SimpleTracker {
    fn id(&self) -> ID {
        self.id
    }

    fn get_tracking_chain(&self) -> TrackingChain {
        TrackingChain::new(self.id, self.chain.clone())
    }

    fn is_dead(&self) -> bool {
        false
    }

    fn signal_no_matching_node(&mut self) {}

    fn add_node(&mut self, node: ChainNode, record: Record) {
        self.record = record;
        self.chain.push(node);
    }

    fn process_frame(
        &mut self,
        frame: &Frame,
        distance_calculators: &mut Vec<CachedDistanceCalculator>,
    ) -> Vec<RecordScore> {
        let distances = self.compute_distances(frame, distance_calculators);

        let mut scores = Vec::new();

        for record_idx in 0..frame.num_records() {
            let score = self.compute_score_record(frame, &distances, record_idx);
            if score > self.config.interest_threshold {
                scores.push(RecordScore::new(record_idx, score));
            }
        }

        // sort in descending order
        scores.sort_unstable_by(|a, b| b.cmp(a));
        scores
    }
}
