use crate::{
    frame::Frame,
    id::ID,
    trackers::{RecordScore, Tracker},
};

/// ScoreBucket
///
/// Specific to a record, it contains a list of score, ID tuples of
/// the trackers that considered it of interest.
///
/// The list is always sorted in descending order of score, so the highest score
/// is always the first element.
pub struct ScoreBucket {
    scores: Vec<(f32, ID)>,
}

impl ScoreBucket {
    pub fn new() -> Self {
        Self { scores: Vec::new() }
    }

    /// Pushes a new score, ID tuple to the bucket.
    ///
    /// This maintains the list sorted in descending order of score.
    pub fn push(&mut self, score: f32, id: ID) {
        for (i, (s, _)) in self.scores.iter().enumerate() {
            if score > *s {
                self.scores.insert(i, (score, id));
                break;
            }
        }
        self.scores.push((score, id));
    }
}

/// ResolvingStrategy
///
/// Responsible to decide which records match to which trackers and to create
/// new trackers.
pub trait ResolvingStrategy {
    /// Resolves the matching records to the trackers for the current frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - The current frame.
    /// * `trackers` - The list of active trackers.
    /// * `buckets` - The list of score buckets,
    ///             each record has a score bucket, it contains
    ///             the score and ID of each tracker that considered it of interest.
    /// * `trackers_scores` - For each tracker, the list of scores for the records of interest.
    ///
    /// # Returns
    /// A list of new trackers.
    fn resolve(
        &mut self,
        frame: &Frame,
        trackers: &mut Vec<Tracker>,
        buckets: Vec<ScoreBucket>,
        trackers_scores: Vec<Vec<RecordScore>>,
    ) -> Vec<Tracker>;
}

/// Resolver
///
/// Responsible for applying the resolving strategy given the trackers scores.
pub struct Resolver {
    resolving_strategy: Box<dyn ResolvingStrategy>,
}

impl Resolver {
    pub fn new(resolving_strategy: Box<dyn ResolvingStrategy>) -> Self {
        Self { resolving_strategy }
    }

    /// Applies the resolving strategy to the trackers scores.
    ///  
    /// # Arguments
    ///
    /// * `frame` - The current frame.
    /// * `trackers` - The list of active trackers.
    /// * `trackers_scores` - For each tracker, the list of scores for the records of interest.
    ///
    /// # Returns
    /// A list of new trackers.
    pub fn resolve(
        &mut self,
        frame: &Frame,
        trackers: &mut Vec<Tracker>,
        trackers_scores: Vec<Vec<RecordScore>>,
    ) -> Vec<Tracker> {
        let mut buckets = (0..frame.num_records())
            .into_iter()
            .map(|_| ScoreBucket::new())
            .collect::<Vec<ScoreBucket>>();

        for (tracker_scores, tracker) in trackers_scores.iter().zip(trackers.iter()) {
            for score in tracker_scores.iter() {
                buckets[score.idx].push(score.score, tracker.id());
            }
        }

        self.resolving_strategy
            .resolve(frame, trackers, buckets, trackers_scores)
    }
}
