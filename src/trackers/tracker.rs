use crate::{
    api::{ChainNode, TrackerDiagnostics, TrackerFrameDiagnostics, TrackerRecordDiagnostics},
    distances::{CachedDistanceCalculator, InternalDistanceMetricConfig},
    frame::{Element, Frame, Record},
    id::{self, ID},
};

use super::{
    tracker_memory::{
        BruteForceMemory, LongShortTermMemory, MedianWordMemory, MostFrequentMemory,
        MultiWordMemory,
    },
    AverageRecordScorer, WeightedAverageRecordScorer, WeightedQuadraticRecordScorer,
};

/// TrackingChain
///
/// Represents a chain of chain nodes.
#[derive(Debug, Clone)]
pub struct TrackingChain {
    pub id: ID,
    pub nodes: Vec<ChainNode>,
}

impl TrackingChain {
    pub fn new(id: ID, nodes: Vec<ChainNode>) -> Self {
        Self { id, nodes }
    }
}

/// RecordScore
///
/// Represents the score of a record.
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct RecordScore {
    pub idx: usize,
    pub score: f32,
}

impl RecordScore {
    pub fn new(idx: usize, score: f32) -> Self {
        Self { score, idx }
    }
}

/// Implementing the `Ord` trait for `RecordScore` to allow sorting.
/// This is valid because score is never NaN.
impl Eq for RecordScore {}

impl Ord for RecordScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.total_cmp(&other.score)
    }
}

#[derive(Debug, Clone)]
pub enum TrackerMemoryConfig {
    BruteForce,
    MostFrequent,
    Median,
    LongShortTerm(Box<TrackerMemoryConfig>),
    MultiWord(Box<TrackerMemoryConfig>, InternalDistanceMetricConfig, f32),
}

#[derive(Debug, Clone)]
pub enum TrackerRecordScorerConfig {
    Average,
    WeightedAverage(Vec<f32>, f32),
    WeightedQuadratic(Vec<f32>, f32),
}

#[derive(Clone)]
pub struct InternalTrackerConfig {
    pub interest_threshold: f32,
    pub limit_no_match_streak: usize,
    pub memory_configs: Vec<TrackerMemoryConfig>,
    pub record_scorer: TrackerRecordScorerConfig,
}

/// TrackerMemory
///
/// Represents the memory of a feature (column) of a tracker.
/// It is responsible for storing the elements that have been seen by the tracker
/// and select the most relevant values to be used in the distance calculation
/// with the next frame.
pub trait TrackerMemory {
    /// Signals that no matching element has been found in the current frame.
    fn signal_no_matching_element(&mut self);

    /// Signals that a matching element has been found in the current frame.
    ///
    /// Updates the memory with the new element.
    fn signal_matching_element(&mut self, element: Element);

    /// Returns the relevant elements according to the memory policy.
    ///
    /// This should not be computationally expensive, computation should be done
    /// on signaling the matching element.
    ///
    /// Elements of type Element::None must not be returned.
    fn get_elements(&self) -> Vec<&Element>;

    /// Returns a new instance of the memory with the default values.
    fn new_default(&self) -> Box<dyn TrackerMemory + Send + Sync>;
}

/// RecordScorer
///
/// Responsible of scoring a record based on the scores of its features.
pub trait RecordScorer {
    fn score(&self, scores: &Vec<Option<f32>>) -> f32;
}

/// Tracker
///
/// Responsible to track an individual through multiple frames.
/// Each tracker produces a single tracking chain.
pub struct Tracker {
    id: ID,
    config: InternalTrackerConfig,
    chain: Vec<ChainNode>,
    memories: Vec<Box<dyn TrackerMemory + Send + Sync>>,
    record_scorer: Box<dyn RecordScorer + Send + Sync>,
    diagnostics: TrackerDiagnostics,
    no_matching_node_counter: usize,
}

impl Tracker {
    pub fn new(config: InternalTrackerConfig) -> Self {
        let id = id::new_id();
        Self {
            id,
            chain: Vec::new(),
            memories: config
                .memory_configs
                .iter()
                .map(|conf| Self::build_tracker_memory(conf.clone()))
                .collect(),
            record_scorer: Self::build_record_scorer(&config.record_scorer),
            config,
            diagnostics: TrackerDiagnostics::new(id),
            no_matching_node_counter: 0,
        }
    }

    fn build_tracker_memory(
        memory_config: TrackerMemoryConfig,
    ) -> Box<dyn TrackerMemory + Send + Sync> {
        match memory_config {
            TrackerMemoryConfig::BruteForce => Box::new(BruteForceMemory::new()),
            TrackerMemoryConfig::MostFrequent => Box::new(MostFrequentMemory::new()),
            TrackerMemoryConfig::Median => Box::new(MedianWordMemory::new()),
            TrackerMemoryConfig::LongShortTerm(memory_config) => Box::new(
                LongShortTermMemory::new(Self::build_tracker_memory(*memory_config)),
            ),
            TrackerMemoryConfig::MultiWord(
                memory_config,
                distance_metric_config,
                threshold_match,
            ) => Box::new(MultiWordMemory::new(
                Self::build_tracker_memory(*memory_config),
                distance_metric_config.make_metric(),
                threshold_match,
            )),
        }
    }

    fn build_record_scorer(
        record_scorer_config: &TrackerRecordScorerConfig,
    ) -> Box<dyn RecordScorer + Send + Sync> {
        match record_scorer_config {
            TrackerRecordScorerConfig::Average => Box::new(AverageRecordScorer::new()),
            TrackerRecordScorerConfig::WeightedAverage(weights, ratio) => {
                Box::new(WeightedAverageRecordScorer::new(weights.clone(), *ratio))
            }
            TrackerRecordScorerConfig::WeightedQuadratic(weights, ratio) => {
                Box::new(WeightedQuadraticRecordScorer::new(weights.clone(), *ratio))
            }
        }
    }

    /// Returns the ID of the tracker
    pub fn id(&self) -> ID {
        self.id
    }

    /// Takes the diagnostics of the tracker.
    ///
    /// This will reset the diagnostics of the tracker.
    pub fn take_diagnostics(&mut self) -> TrackerDiagnostics {
        std::mem::replace(&mut self.diagnostics, TrackerDiagnostics::new(self.id))
    }

    /// Builds the tracking chain for the tracker at this time.
    pub fn get_tracking_chain(&self) -> TrackingChain {
        TrackingChain::new(self.id, self.chain.clone())
    }

    /// Returns the memory elements for a feature.
    pub fn get_memory_elements(&self, feature_idx: usize) -> Vec<&Element> {
        self.memories[feature_idx].get_elements()
    }

    /// Returns if the tracker is considered dead.
    ///
    /// This happens when no matching records have been found for a certain amount of frames.
    /// It is useful to reduce the number of trackers that are being processed.
    pub fn is_dead(&self) -> bool {
        self.no_matching_node_counter > self.config.limit_no_match_streak
    }

    /// Signals that no matching node has been found in the current frame.
    pub fn signal_no_matching_node(&mut self) {
        self.no_matching_node_counter += 1;
        for memory in self.memories.iter_mut() {
            memory.signal_no_matching_element();
        }
    }

    /// Signals that a matching node has been found in the current frame
    /// and add it to the tracker's chain.
    ///
    /// The matching record is also provided to update the tracker's memory.
    pub fn signal_matching_node(&mut self, node: ChainNode, record: Record) {
        self.chain.push(node);
        self.no_matching_node_counter = 0;
        for idx in 0..record.size() {
            self.memories[idx].signal_matching_element(record.element(idx).clone());
        }
    }

    /// Saves the current elements of the memories the tracker to the frame diagnostics.
    fn save_memory_to_diagnostics(&self, diagnostics: &mut TrackerFrameDiagnostics) {
        let mut memories = Vec::new();
        for memory in self.memories.iter() {
            let mut memory_strings = Vec::new();
            for element in memory.get_elements().iter_mut() {
                match element {
                    Element::MultiWords(words) => {
                        for word in words {
                            memory_strings.push(word.raw.clone());
                        }
                    }
                    Element::Word(word) => {
                        memory_strings.push(word.raw.clone());
                    }
                    Element::None => {}
                }
            }
            memories.push(memory_strings);
        }

        diagnostics.memory = memories;
    }

    /// Computes the distances between the tracker's memory and the frame's records.
    ///
    /// Returns a matrix of distances, with one vector per record and one element per feature.
    fn compute_distances(
        &self,
        frame: &Frame,
        distance_calculators: &mut Vec<CachedDistanceCalculator>,
    ) -> Vec<Vec<Option<f32>>> {
        let mut distances = (0..frame.num_records())
            .map(|_| (0..frame.num_features()).map(|_| None).collect())
            .collect::<Vec<Vec<Option<f32>>>>();

        for feature_idx in 0..frame.num_features() {
            let distance_calculator = &mut distance_calculators[feature_idx];
            let own_elements = self.memories[feature_idx].get_elements();

            for (record_idx, element) in frame.column(feature_idx).iter().enumerate() {
                let mut max_dist: Option<f32> = None;
                for own_element in own_elements.iter() {
                    let dist = distance_calculator.get_dist(own_element, element);
                    if let Some(dist) = dist {
                        max_dist = max_dist.map(|d| d.max(dist)).or(Some(dist));
                    }
                }
                distances[record_idx][feature_idx] = max_dist;
            }
        }

        distances
    }

    /// Processes a frame, that is computes the distances between the tracker's memory
    /// and the frame's records to find the "best" records.
    ///
    /// Returns a list of record scores, for the records considered of interest by the tracker.
    /// The list is sorted in descending order of score.
    pub fn process_frame(
        &mut self,
        frame: &Frame,
        distance_calculators: &mut Vec<CachedDistanceCalculator>,
    ) -> Vec<RecordScore> {
        let distances = self.compute_distances(frame, distance_calculators);

        let mut scores = Vec::new();
        let mut frame_diagnostics = TrackerFrameDiagnostics::new(frame.idx());

        for record_idx in 0..frame.num_records() {
            let score = self.record_scorer.score(&distances[record_idx]);
            if score > self.config.interest_threshold {
                scores.push(RecordScore::new(record_idx, score));
                frame_diagnostics
                    .records
                    .push(TrackerRecordDiagnostics::new(
                        record_idx,
                        score,
                        distances[record_idx].clone(),
                    ));
            }
        }

        self.save_memory_to_diagnostics(&mut frame_diagnostics);

        self.diagnostics.frames.push(frame_diagnostics);

        // sort in descending order
        scores.sort_unstable_by(|a, b| b.cmp(a));
        scores
    }
}
