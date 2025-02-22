use crate::{
    distances::CachedDistanceCalculator,
    frame::{Frame, Record},
    id::ID,
};

#[derive(Debug, Clone, Copy)]
pub struct TrackingNode {
    pub frame_idx: usize,
    pub record_idx: usize,
}

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

/// For RecordScore reflexivity holds because score is never NaN
impl Eq for RecordScore {}

impl Ord for RecordScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.total_cmp(&other.score)
    }
}

pub trait Tracker<'a> {
    fn id(&self) -> ID;

    fn get_tracking_chain(&self) -> &Vec<TrackingNode>;

    fn is_dead(&self) -> bool;

    fn signal_no_matching_node(&mut self);

    fn add_node(&mut self, node: TrackingNode, record: Record<'a>);

    fn process_frame(
        &mut self,
        frame: &Frame<'a>,
        distance_calculators: &mut Vec<CachedDistanceCalculator<'a>>,
    ) -> Vec<RecordScore>;
}
