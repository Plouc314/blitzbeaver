use crate::{
    api::ChainNode,
    frame::Frame,
    trackers::{RecordScore, Tracker},
};

use super::{ResolvingStrategy, ScoreBucket};

pub struct SimpleResolvingStrategy {}

impl ResolvingStrategy for SimpleResolvingStrategy {
    fn resolve(
        &mut self,
        frame: &Frame,
        trackers: &mut Vec<Box<dyn Tracker>>,
        buckets: Vec<ScoreBucket>,
        trackers_scores: Vec<Vec<RecordScore>>,
    ) -> Vec<Box<dyn Tracker>> {
        for tracker_idx in 0..trackers.len() {
            let scores = &trackers_scores[tracker_idx];
            if scores.len() == 0 {
                trackers[tracker_idx].signal_no_matching_node();
            } else {
                let score = scores[0];
                let node = ChainNode {
                    frame_idx: frame.idx(),
                    record_idx: score.idx,
                };
                trackers[tracker_idx].add_node(node, frame.record(score.idx));
            }
        }

        Vec::new()
    }
}
