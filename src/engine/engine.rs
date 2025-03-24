use std::{collections::HashMap, sync::Arc};

use crate::{
    api::{ChainNode, Diagnostics, ResolvingDiagnostics},
    distances::CachedDistanceCalculator,
    frame::Frame,
    id::ID,
    resolvers::Resolver,
    trackers::{InternalTrackerConfig, RecordScore, Tracker, TrackingChain},
};

use super::{exclusive_shared::ExclusiveShared, worker::TrackingWorkerHandler};

pub struct EngineConfig {
    pub num_threads: usize,
    pub tracker_config: InternalTrackerConfig,
}

/// TrackingEngine
///
/// The main engine that orchestrates the tracking process.
///
/// It is responsible for managing the trackers, workers, and the resolving process.
pub struct TrackingEngine {
    frames: Arc<Vec<Frame>>,
    config: EngineConfig,
    workers: Vec<TrackingWorkerHandler>,
    resolver: Resolver,
    trackers: HashMap<ID, ExclusiveShared<Tracker>>,
    diagnostics: Diagnostics,
    dead_tracking_chains: Vec<TrackingChain>,
    next_frame_idx: usize,
}

impl TrackingEngine {
    pub fn new(
        frames: Vec<Frame>,
        config: EngineConfig,
        resolver: Resolver,
        distance_calculators: Vec<CachedDistanceCalculator>,
    ) -> Self {
        let frames = Arc::new(frames);
        let trackers = Self::build_trackers(&frames, &config);
        let workers = Self::build_workers(&frames, &config, &distance_calculators);

        let mut engine = Self {
            frames,
            config,
            workers,
            resolver,
            trackers: HashMap::new(),
            diagnostics: Diagnostics::new(),
            dead_tracking_chains: Vec::new(),
            next_frame_idx: 1,
        };
        engine.add_new_trackers(trackers);

        engine
    }

    /// Builds the workers given the frames and configuration.
    ///
    /// The trackers will be added at a later stage.
    fn build_workers(
        frames: &Arc<Vec<Frame>>,
        config: &EngineConfig,
        distance_calculators: &Vec<CachedDistanceCalculator>,
    ) -> Vec<TrackingWorkerHandler> {
        let n_workers = config.num_threads - 1;
        let mut workers = Vec::new();
        for _ in 0..n_workers {
            let worker = TrackingWorkerHandler::new(
                Arc::clone(frames),
                HashMap::new(),
                distance_calculators.clone(),
            );
            workers.push(worker);
        }
        workers
    }

    /// Builds the initial trackers from the first frame.
    ///
    /// Each record in the first frame will be used to initialize a tracker.
    fn build_trackers(frames: &Arc<Vec<Frame>>, config: &EngineConfig) -> Vec<Tracker> {
        let frame = &frames[0];

        let mut trackers = Vec::new();

        for i in 0..frame.num_records() {
            let mut tracker = Tracker::new(config.tracker_config.clone());
            tracker.signal_matching_node(
                ChainNode {
                    frame_idx: 0,
                    record_idx: i,
                },
                frame.record(i),
            );
            trackers.push(tracker);
        }

        trackers
    }

    /// Returns the frames
    pub fn frames(&self) -> &Vec<Frame> {
        &self.frames
    }

    /// Takes the diagnostics
    ///
    /// This will reset the diagnostics.
    pub fn take_diagnostics(&mut self) -> Diagnostics {
        std::mem::replace(&mut self.diagnostics, Diagnostics::new())
    }

    /// Checks for dead trackers and removes them from the engine
    /// and workers.
    ///
    /// Collect the diagnostics from the dead trackers.
    fn remove_dead_trackers(&mut self) {
        let mut removed_ids = Vec::new();
        for (id, tracker) in self.trackers.iter_mut() {
            if tracker.is_dead() {
                self.diagnostics
                    .trackers
                    .insert(*id, tracker.exclusive().take_diagnostics());

                self.dead_tracking_chains.push(tracker.get_tracking_chain());
                removed_ids.push(*id);
            }
        }

        log::debug!(
            "frame: {} dead trackers: {}",
            self.next_frame_idx,
            removed_ids.len()
        );

        for id in removed_ids.iter() {
            self.trackers.remove(id);
        }

        for worker in self.workers.iter_mut() {
            worker.remove_trackers(removed_ids.clone());
        }
    }

    fn add_trackers_to_worker(
        worker: &mut TrackingWorkerHandler,
        trackers: &[ExclusiveShared<Tracker>],
    ) {
        let mut added_trackers = HashMap::new();
        for tracker in trackers {
            let tracker = ExclusiveShared::clone(tracker);
            added_trackers.insert(tracker.id(), tracker);
        }
        worker.add_trackers(added_trackers);
    }

    /// Adds trackers to the engine
    ///
    /// Distributes the trackers among the workers.
    fn add_new_trackers(&mut self, trackers: Vec<Tracker>) {
        let trackers: Vec<ExclusiveShared<Tracker>> = trackers
            .into_iter()
            .map(|t| ExclusiveShared::new(t))
            .collect();

        // computes current average number of trackers per worker
        let avg_tracker_count =
            self.workers.iter().map(|w| w.num_trackers()).sum::<usize>() / self.workers.len();

        // sort workers by number of trackers to first refill
        // workers with less trackers
        self.workers.sort_by_key(|w| w.num_trackers());

        let mut idx = 0;

        // add trackers to workers with less than average number of trackers
        for worker in self.workers.iter_mut() {
            // break if all trackers have been added
            if idx == trackers.len() {
                break;
            }
            // break if worker already has at least the average number of trackers
            if avg_tracker_count <= worker.num_trackers() {
                break;
            }
            let n_addition = usize::min(
                avg_tracker_count - worker.num_trackers(),
                trackers.len() - idx,
            );

            Self::add_trackers_to_worker(worker, &trackers[idx..idx + n_addition]);

            idx += n_addition;
        }

        // split remaining trackers equally among workers
        let n_addition = (trackers.len() - idx) / self.workers.len();
        for worker in self.workers.iter_mut() {
            Self::add_trackers_to_worker(worker, &trackers[idx..idx + n_addition]);
            idx += n_addition;
        }

        // add remaining trackers to the first worker
        if idx < trackers.len() {
            Self::add_trackers_to_worker(&mut self.workers[0], &trackers[idx..]);
        }

        // add trackers to the engine tracker list
        self.trackers
            .extend(trackers.into_iter().map(|t| (t.id(), t)));
    }

    /// Executes the resolving process
    fn process_resolving(
        &mut self,
        tracker_scores: HashMap<ID, Vec<RecordScore>>,
    ) -> (Vec<Tracker>, ResolvingDiagnostics) {
        let mut trackers: Vec<ExclusiveShared<Tracker>> = self
            .trackers
            .iter()
            .map(|(_, t)| ExclusiveShared::clone(t))
            .collect();

        let mut scores = Vec::new();
        for tracker in trackers.iter() {
            let id = tracker.id();
            let tracker_scores = tracker_scores.get(&id).unwrap();
            scores.push(tracker_scores.clone());
        }

        self.resolver.resolve(
            &self.frames[self.next_frame_idx],
            self.config.tracker_config.clone(),
            &mut trackers,
            scores,
        )
    }

    /// Processes the next frame
    pub fn process_next_frame(&mut self) {
        for worker in self.workers.iter_mut() {
            worker.process_frame(self.next_frame_idx);
        }

        let mut trackers_scores = HashMap::with_capacity(self.trackers.len());

        for worker in self.workers.iter() {
            let scores = worker.wait_scores();
            trackers_scores.extend(scores.into_iter());
        }

        let (new_trackers, resolving_diagnostics) = self.process_resolving(trackers_scores);

        self.diagnostics.resolvings.push(resolving_diagnostics);

        self.remove_dead_trackers();
        self.add_new_trackers(new_trackers);

        self.next_frame_idx += 1;
    }

    /// Collects the state of the trackers
    ///
    /// This includes the tracking chains and diagnostics.
    fn collect_trackers_state(&mut self) -> Vec<TrackingChain> {
        let mut tracking_chains = self.dead_tracking_chains.clone();
        for (_, tracker) in self.trackers.iter_mut() {
            tracking_chains.push(tracker.get_tracking_chain());
            self.diagnostics
                .trackers
                .insert(tracker.id(), tracker.exclusive().take_diagnostics());
        }
        tracking_chains
    }

    /// Stops the engine
    ///
    /// Stops all workers, collects the diagnostics and tracking chains.
    ///
    /// Returns the tracking chains.
    pub fn stop(&mut self) -> Vec<TrackingChain> {
        for worker in self.workers.iter() {
            worker.stop();
        }
        self.collect_trackers_state()
    }
}
