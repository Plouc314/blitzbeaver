use crate::{
    api::ChainNode,
    distances::CachedDistanceCalculator,
    frame::Frame,
    resolvers::Resolver,
    trackers::{Tracker, TrackingChain},
};

pub struct TrackingEngine<F>
where
    F: Fn() -> Box<dyn Tracker>,
{
    frames: Vec<Frame>,
    resolver: Resolver,
    distance_calculators: Vec<CachedDistanceCalculator>,
    trackers: Vec<Box<dyn Tracker>>,
    tracker_builder: F,
    dead_tracking_chains: Vec<TrackingChain>,
    next_frame_idx: usize,
}

impl<F> TrackingEngine<F>
where
    F: Fn() -> Box<dyn Tracker>,
{
    pub fn new(
        frames: Vec<Frame>,
        resolver: Resolver,
        distance_calculators: Vec<CachedDistanceCalculator>,
        tracker_builder: F,
    ) -> Self {
        Self {
            frames,
            resolver,
            distance_calculators,
            trackers: Vec::new(),
            tracker_builder,
            dead_tracking_chains: Vec::new(),
            next_frame_idx: 1,
        }
    }

    pub fn frames(&self) -> &Vec<Frame> {
        &self.frames
    }

    pub fn initialize_trackers(&mut self) {
        let frame = self
            .frames
            .first()
            .expect("there must be at least 2 frames");

        for i in 0..frame.num_records() {
            let mut tracker = (self.tracker_builder)();
            tracker.add_node(
                ChainNode {
                    frame_idx: 0,
                    record_idx: i,
                },
                frame.record(i),
            );
            self.trackers.push(tracker);
        }
    }

    fn remove_dead_trackers(&mut self) {
        let mut i = 0;
        while i < self.trackers.len() {
            if self.trackers[i].is_dead() {
                let tracker = self.trackers.swap_remove(i);
                self.dead_tracking_chains.push(tracker.get_tracking_chain());
            } else {
                i += 1;
            }
        }
    }

    pub fn process_next_frame(&mut self) {
        let prev_frame = self
            .frames
            .get(self.next_frame_idx - 1)
            .expect("invalid next_frame_idx");
        let next_frame = self
            .frames
            .get(self.next_frame_idx)
            .expect("invalid next_frame_idx");

        for feature_idx in 0..next_frame.num_features() {
            let distance_calculator = &mut self.distance_calculators[feature_idx];
            distance_calculator.clear_cache();
            distance_calculator.precompute(
                prev_frame.column(feature_idx),
                next_frame.column(feature_idx),
            );
        }

        let mut trackers_scores = Vec::with_capacity(self.trackers.len());

        for tracker in self.trackers.iter_mut() {
            let scores = tracker.process_frame(next_frame, &mut self.distance_calculators);

            trackers_scores.push(scores);
        }

        let mut new_trackers =
            self.resolver
                .resolve(next_frame, &mut self.trackers, trackers_scores);

        self.remove_dead_trackers();
        self.trackers.append(&mut new_trackers);

        self.next_frame_idx += 1;
    }

    pub fn collect_tracking_chains(&mut self) -> Vec<TrackingChain> {
        let mut tracking_chains = self.dead_tracking_chains.clone();
        for tracker in self.trackers.iter() {
            tracking_chains.push(tracker.get_tracking_chain());
        }
        tracking_chains
    }
}
