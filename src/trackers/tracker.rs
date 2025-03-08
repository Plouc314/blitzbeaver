use crate::{
    api::{ChainNode, TrackerConfig},
    distances::CachedDistanceCalculator,
    frame::{Element, Frame, Record},
    id::{self, ID},
};

use super::{
    tracker_memory::{BruteForceMemory, LongShortTermMemory, MedianWordMemory, MostFrequentMemory},
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
pub enum TrackerMemoryStrategy {
    BruteForce,
    MostFrequent,
    Median,
    LSBruteForce,
    LSMostFrequent,
    LSMedian,
}

#[derive(Debug, Clone)]
pub enum TrackerRecordScorer {
    Average,
    WeightedAverage(Vec<f32>),
    WeightedQuadratic(Vec<f32>),
}

#[derive(Clone)]
pub struct InternalTrackerConfig {
    pub interest_threshold: f32,
    pub memory_strategy: TrackerMemoryStrategy,
    pub record_scorer: TrackerRecordScorer,
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
}

pub trait RecordScorer {
    fn score(&self, scores: &Vec<f32>) -> f32;
}

/// Tracker
///
/// Responsible to track an individual through multiple frames.
/// Each tracker produces a single tracking chain.
pub struct Tracker {
    id: ID,
    config: InternalTrackerConfig,
    chain: Vec<ChainNode>,
    memories: Vec<Box<dyn TrackerMemory + Send>>,
    record_scorer: Box<dyn RecordScorer + Send>,
}

impl Tracker {
    pub fn new(config: InternalTrackerConfig, num_features: usize) -> Self {
        Self {
            id: id::new_id(),

            chain: Vec::new(),
            memories: (0..num_features)
                .map(|_| Self::build_tracker_memory(config.memory_strategy.clone()))
                .collect(),
            record_scorer: Self::build_record_scorer(&config.record_scorer),
            config,
        }
    }

    fn build_tracker_memory(
        memory_strategy: TrackerMemoryStrategy,
    ) -> Box<dyn TrackerMemory + Send> {
        match memory_strategy {
            TrackerMemoryStrategy::BruteForce => Box::new(BruteForceMemory::new()),
            TrackerMemoryStrategy::MostFrequent => Box::new(MostFrequentMemory::new()),
            TrackerMemoryStrategy::Median => Box::new(MedianWordMemory::new()),
            TrackerMemoryStrategy::LSBruteForce => {
                Box::new(LongShortTermMemory::new(Box::new(BruteForceMemory::new())))
            }
            TrackerMemoryStrategy::LSMostFrequent => Box::new(LongShortTermMemory::new(Box::new(
                MostFrequentMemory::new(),
            ))),
            TrackerMemoryStrategy::LSMedian => {
                Box::new(LongShortTermMemory::new(Box::new(MedianWordMemory::new())))
            }
        }
    }

    fn build_record_scorer(
        record_scorer_config: &TrackerRecordScorer,
    ) -> Box<dyn RecordScorer + Send> {
        match record_scorer_config {
            TrackerRecordScorer::Average => Box::new(AverageRecordScorer::new()),
            TrackerRecordScorer::WeightedAverage(weights) => {
                Box::new(WeightedAverageRecordScorer::new(weights.clone()))
            }
            TrackerRecordScorer::WeightedQuadratic(weights) => {
                Box::new(WeightedQuadraticRecordScorer::new(weights.clone()))
            }
        }
    }

    /// Returns the ID of the tracker
    pub fn id(&self) -> ID {
        self.id
    }

    /// Builds the tracking chain for the tracker at this time.
    pub fn get_tracking_chain(&self) -> TrackingChain {
        TrackingChain::new(self.id, self.chain.clone())
    }

    /// Returns if the tracker is considered dead.
    ///
    /// This happens when no matching records have been found for a certain amount of frames.
    /// It is useful to reduce the number of trackers that are being processed.
    pub fn is_dead(&self) -> bool {
        false
    }

    /// Signals that no matching node has been found in the current frame.
    pub fn signal_no_matching_node(&mut self) {}

    /// Signals that a matching node has been found in the current frame
    /// and add it to the tracker's chain.
    ///
    /// The matching record is also provided to update the tracker's memory.
    pub fn signal_matching_node(&mut self, node: ChainNode, record: Record) {
        self.chain.push(node);
        for idx in 0..record.size() {
            self.memories[idx].signal_matching_element(record.element(idx).clone());
        }
    }

    /// Computes the distances between the tracker's memory and the frame's records.
    ///
    /// Returns a matrix of distances, with one vector per record and one element per feature.
    fn compute_distances(
        &self,
        frame: &Frame,
        distance_calculators: &mut Vec<CachedDistanceCalculator>,
    ) -> Vec<Vec<f32>> {
        let mut distances = (0..frame.num_records())
            .map(|_| (0..frame.num_features()).map(|_| 0.0).collect())
            .collect::<Vec<Vec<f32>>>();

        for feature_idx in 0..frame.num_features() {
            let distance_calculator = &mut distance_calculators[feature_idx];
            let own_elements = self.memories[feature_idx].get_elements();

            for (record_idx, element) in frame.column(feature_idx).iter().enumerate() {
                let mut max_dist: f32 = 0.0;
                for own_element in own_elements.iter() {
                    let dist = distance_calculator.get_dist(own_element, element);
                    max_dist = max_dist.max(dist);
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

        for record_idx in 0..frame.num_records() {
            let score = self.record_scorer.score(&distances[record_idx]);
            if score > self.config.interest_threshold {
                scores.push(RecordScore::new(record_idx, score));
            }
        }

        // sort in descending order
        scores.sort_unstable_by(|a, b| b.cmp(a));
        scores
    }
}
