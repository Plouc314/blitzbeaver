use crate::{
    api::ChainNode,
    frame::Frame,
    trackers::{InternalTrackerConfig, RecordScore, Tracker},
};

use super::{ResolvingStrategy, ScoreBucket};

pub struct SimpleResolvingStrategy {}

impl ResolvingStrategy for SimpleResolvingStrategy {
    fn resolve(
        &mut self,
        frame: &Frame,
        tracker_config: InternalTrackerConfig,
        trackers: &mut Vec<Tracker>,
        buckets: Vec<ScoreBucket>,
        trackers_scores: Vec<Vec<RecordScore>>,
    ) -> Vec<Tracker> {
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
                trackers[tracker_idx].signal_matching_node(node, frame.record(score.idx));
            }
        }

        Vec::new()
    }
}
