use crate::{
    distances::CachedDistanceCalculator,
    frame::{Frame, Record},
    id::ID,
};

/// TrackingNode
///
/// References a record in a frame.
///
/// Note: this doesn't hold the record itself, but only the indices to access it.
#[derive(Debug, Clone, Copy)]
pub struct TrackingNode {
    pub frame_idx: usize,
    pub record_idx: usize,
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

/// Tracker
///
/// Responsible to track an individual through multiple frames.
/// Each tracker produces a single tracking chain.
pub trait Tracker<'a> {
    /// Returns the ID of the tracker
    fn id(&self) -> ID;

    /// Returns the tracking chain of the tracker
    fn get_tracking_chain(&self) -> &Vec<TrackingNode>;

    /// Returns if the tracker is considered dead.
    ///
    /// This should happen when no matching records have been found for a certain amount of frames.
    /// It is useful to reduce the number of trackers that are being processed.
    fn is_dead(&self) -> bool;

    /// Signals that no matching node has been found in the current frame.
    fn signal_no_matching_node(&mut self);

    /// Adds a node to the tracking chain.
    ///
    /// This also signals that a matching node has been found in the current frame.
    /// The matching record is also provided to update the tracker's memory.
    ///
    /// In case no matching node is found, the `signal_no_matching_node` method is called instead.
    fn add_node(&mut self, node: TrackingNode, record: Record<'a>);

    /// Processes a frame, that is computes the distances between the tracker's memory
    /// and the frame's records to find the "best" records.
    ///
    /// Returns a list of record scores, for the records considered of interest by the tracker.
    /// The list must be sorted in descending order of score.
    fn process_frame(
        &mut self,
        frame: &Frame<'a>,
        distance_calculators: &mut Vec<CachedDistanceCalculator<'a>>,
    ) -> Vec<RecordScore>;
}
