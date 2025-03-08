use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    usize,
};

use crate::{
    api::ChainNode,
    frame::Frame,
    id::ID,
    trackers::{InternalTrackerConfig, RecordScore, Tracker},
};

use super::{ResolvingStrategy, ScoreBucket};

enum TrackerStatus {
    Resolved(usize),
    StandBy,
    WontResolve,
}

/// BestMatchResolvingStrategy
///
/// The best match strategy tries to attribute the most relevant record
/// to each tracker, with the following properties:
/// - At most one tracker matches with a certain record.
/// - Each tracker matches with at most one record.
///
/// In short it doesn't allow conflicts or divergences.
///
/// Each tracker goes through its candidate scores. For a given candidate record, it checks the bucket of scores.
/// * If the tracker’s own ID appears as the top candidate for that record, it immediately resolves that tracker with the record.
/// * If it finds that another tracker (with a higher score for that record) is already resolved, it skips that candidate.
/// * If it encounters a competing tracker that is not yet resolved, it returns a “StandBy” status, meaning the tracker is waiting
///   for a decision on that competing tracker.
/// * If it goes through all candidates without resolving, it returns a “WontResolve” status.
///
/// The main loop repeatedly processes only those trackers that are in the standby state until none remain. In each iteration,
/// trackers either get resolved, are marked as “WontResolve” (if no candidate record meets the criteria), or remain on standby.
///
/// After the loop, it determines which records have not been matched to any tracker and creates new trackers for them.
pub struct BestMatchResolvingStrategy {}

impl BestMatchResolvingStrategy {
    fn resolve_tracker(
        &self,
        tracker_id: ID,
        tracker_scores: &Vec<RecordScore>,
        buckets: &Vec<ScoreBucket>,
        resolved_trackers: &HashMap<ID, usize>,
    ) -> TrackerStatus {
        for tracker_score in tracker_scores {
            let bucket = &buckets[tracker_score.idx];
            for (_, id) in bucket.scores() {
                // if the best score for this record is the one of the
                // tracker => the tracker is resolved with this record
                if *id == tracker_id {
                    return TrackerStatus::Resolved(tracker_score.idx);
                }
                // check if the tracker which has a better score for
                // this record is already resolved
                match resolved_trackers.get(id) {
                    Some(record_idx) => {
                        // the tracker is already resolved
                        // check if it is resolved with this record
                        // => can't resolve with this record, go to
                        // the next tracker's score
                        if *record_idx == tracker_score.idx {
                            break;
                        }
                        // it is resolved with another record
                        // => can ignore it and check the next score for
                        // this record
                        continue;
                    }
                    None => {
                        // the tracker is not yet resolved, this tracker
                        // is in standby until it is
                        return TrackerStatus::StandBy;
                    }
                }
            }
        }
        TrackerStatus::WontResolve
    }

    /// Resolve the trackers in the given list
    /// of trackers indexes.
    ///
    /// Return the indexes of the trackers that are still in standby
    fn resolve_trackers(
        &self,
        frame: &Frame,
        trackers: &Vec<Arc<Mutex<Tracker>>>,
        trackers_scores: &Vec<Vec<RecordScore>>,
        buckets: &Vec<ScoreBucket>,
        trackers_idx: &Vec<usize>,
        resolved_trackers: &mut HashMap<ID, usize>,
    ) -> Vec<usize> {
        let mut standby_idxs = Vec::new();
        for tracker_idx in trackers_idx {
            let tracker = &mut trackers[*tracker_idx].lock().unwrap();
            let tracker_scores = &trackers_scores[*tracker_idx];
            match self.resolve_tracker(tracker.id(), tracker_scores, &buckets, &resolved_trackers) {
                TrackerStatus::Resolved(record_idx) => {
                    // the tracker is resolved with this record
                    // update the resolved_trackers map and signal
                    // the tracker
                    resolved_trackers.insert(tracker.id(), record_idx);
                    tracker.signal_matching_node(
                        ChainNode::new(frame.idx(), record_idx),
                        frame.record(record_idx),
                    );
                }
                TrackerStatus::StandBy => {
                    // add the tracker to the standby list
                    // for the next iteration
                    standby_idxs.push(*tracker_idx);
                }
                TrackerStatus::WontResolve => {
                    // the tracker can't be resolved with any record
                    // still add it to the resolved_trackers with a special
                    // value, this is useful to ignore its scores when resolving
                    // standby trackers
                    resolved_trackers.insert(tracker.id(), usize::MAX);
                    tracker.signal_no_matching_node();
                }
            }
        }
        standby_idxs
    }
}

impl ResolvingStrategy for BestMatchResolvingStrategy {
    fn resolve(
        &mut self,
        frame: &Frame,
        tracker_config: InternalTrackerConfig,
        mut trackers: &Vec<Arc<Mutex<Tracker>>>,
        buckets: Vec<ScoreBucket>,
        trackers_scores: Vec<Vec<RecordScore>>,
    ) -> Vec<Tracker> {
        let mut resolved_trackers = HashMap::new();
        let mut trackers_idx: Vec<usize> = (0..trackers.len()).collect();

        while trackers_idx.len() > 0 {
            let standby_idxs = self.resolve_trackers(
                frame,
                trackers,
                &trackers_scores,
                &buckets,
                &trackers_idx,
                &mut resolved_trackers,
            );

            // check for progress
            if trackers_idx.len() == standby_idxs.len() {
                log::warn!(
                    "[resolving:best-match] resolving failed for {} trackers",
                    trackers_idx.len()
                );
                break;
            }
            trackers_idx = standby_idxs;
        }

        // build a map that indicate for each record if it matched with some tracker
        let mut records_match: Vec<bool> = (0..buckets.len()).map(|_| false).collect();
        for (_, idx) in resolved_trackers.iter() {
            if *idx == usize::MAX {
                continue;
            }
            records_match[*idx] = true;
        }

        // build new trackers
        let mut new_trackers = Vec::new();

        for record_idx in 0..buckets.len() {
            if !records_match[record_idx] {
                let mut new_tracker = Tracker::new(tracker_config.clone(), frame.num_features());
                new_tracker.signal_matching_node(
                    ChainNode::new(frame.idx(), record_idx),
                    frame.record(record_idx),
                );
                new_trackers.push(new_tracker);
            }
        }

        new_trackers
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        frame::Element,
        resolvers::Resolver,
        trackers::{TrackerMemoryStrategy, TrackerRecordScorer},
    };

    use super::*;

    fn build_frame(num_records: usize, num_features: usize) -> Frame {
        Frame::new(
            0,
            (0..num_features)
                .map(|_| (0..num_records).map(|_| Element::None).collect())
                .collect(),
        )
    }

    /// Resolve the trackers with the given scores.
    ///
    /// Return the lists of trackers and new trackers.
    fn resolve(
        num_records: usize,
        num_features: usize,
        trackers_scores: Vec<Vec<RecordScore>>,
    ) -> (Vec<Arc<Mutex<Tracker>>>, Vec<Arc<Mutex<Tracker>>>) {
        let strategy = BestMatchResolvingStrategy {};

        let mut resolver = Resolver::new(Box::new(strategy));

        let tracker_config = InternalTrackerConfig {
            interest_threshold: 0.7,
            memory_strategy: TrackerMemoryStrategy::BruteForce,
            record_scorer: TrackerRecordScorer::Average,
        };

        let mut trackers: Vec<Arc<Mutex<Tracker>>> = trackers_scores
            .iter()
            .map(|_| {
                Arc::new(Mutex::new(Tracker::new(
                    tracker_config.clone(),
                    num_features,
                )))
            })
            .collect();

        let frame = build_frame(num_records, num_features);

        let new_trackers = resolver.resolve(&frame, tracker_config, &trackers, trackers_scores);
        (
            trackers,
            new_trackers
                .into_iter()
                .map(|t| Arc::new(Mutex::new(t)))
                .collect(),
        )
    }

    /// Check that the tracker didn't match with any record
    fn tracker_no_match(tracker: &Arc<Mutex<Tracker>>) -> bool {
        tracker
            .lock()
            .unwrap()
            .get_tracking_chain()
            .nodes
            .is_empty()
    }

    /// Check if the tracker is matched with the given record index.
    fn tracker_matched_with(tracker: &Arc<Mutex<Tracker>>, record_idx: usize) -> bool {
        match tracker.lock().unwrap().get_tracking_chain().nodes.first() {
            Some(node) => node.record_idx == record_idx,
            None => false,
        }
    }

    #[test]
    fn test_ideal_setup() {
        let num_records = 3;
        let num_features = 3;
        let trackers_scores = vec![
            vec![
                RecordScore { idx: 0, score: 0.8 },
                RecordScore { idx: 1, score: 0.6 },
                RecordScore { idx: 2, score: 0.5 },
            ],
            vec![
                RecordScore { idx: 0, score: 0.6 },
                RecordScore { idx: 1, score: 0.8 },
                RecordScore { idx: 2, score: 0.5 },
            ],
            vec![
                RecordScore { idx: 0, score: 0.5 },
                RecordScore { idx: 1, score: 0.6 },
                RecordScore { idx: 2, score: 0.8 },
            ],
        ];

        let (trackers, new_trackers) = resolve(num_records, num_features, trackers_scores);

        assert!(new_trackers.is_empty());
        assert_eq!(trackers.len(), 3);

        assert!(tracker_matched_with(&trackers[0], 0));
        assert!(tracker_matched_with(&trackers[1], 1));
        assert!(tracker_matched_with(&trackers[2], 2));
    }

    #[test]
    fn test_new_trackers() {
        let num_records = 3;
        let num_features = 3;
        let trackers_scores = vec![vec![], vec![], vec![]];

        let (trackers, new_trackers) = resolve(num_records, num_features, trackers_scores);

        assert_eq!(new_trackers.len(), 3);
        assert_eq!(trackers.len(), 3);

        assert!(tracker_matched_with(&new_trackers[0], 0));
        assert!(tracker_matched_with(&new_trackers[1], 1));
        assert!(tracker_matched_with(&new_trackers[2], 2));
    }

    #[test]
    fn test_suboptimal_setup() {
        let num_records = 3;
        let num_features = 3;
        let trackers_scores = vec![
            // tracker 0 doesn't match
            vec![
                RecordScore { idx: 0, score: 0.6 },
                RecordScore { idx: 1, score: 0.5 },
                RecordScore { idx: 2, score: 0.3 },
            ],
            // tracker 1 match with record 0
            vec![
                RecordScore { idx: 0, score: 0.8 },
                RecordScore { idx: 1, score: 0.6 },
                RecordScore { idx: 2, score: 0.5 },
            ],
            // tracker 2 match with record 1 (suboptimal)
            vec![
                RecordScore { idx: 0, score: 0.7 },
                RecordScore { idx: 1, score: 0.6 },
                RecordScore { idx: 2, score: 0.5 },
            ],
            // tracker 3 match with record 2 (suboptimal)
            vec![
                RecordScore { idx: 0, score: 0.6 },
                RecordScore { idx: 1, score: 0.5 },
                RecordScore { idx: 2, score: 0.4 },
            ],
        ];

        let (trackers, new_trackers) = resolve(num_records, num_features, trackers_scores);

        assert!(new_trackers.is_empty());
        assert_eq!(trackers.len(), 4);

        assert!(tracker_no_match(&trackers[0]));
        assert!(tracker_matched_with(&trackers[1], 0));
        assert!(tracker_matched_with(&trackers[2], 1));
        assert!(tracker_matched_with(&trackers[3], 2));
    }
}
