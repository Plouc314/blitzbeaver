use crate::{
    api::ResolvingDiagnostics,
    engine::ExclusiveShared,
    frame::Frame,
    histogram::Histogram,
    id::ID,
    trackers::{InternalTrackerConfig, RecordScore, Tracker},
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

    /// Returns the list of scores, ID tuples.
    pub fn scores(&self) -> &Vec<(f32, ID)> {
        &self.scores
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
        tracker_config: InternalTrackerConfig,
        trackers: &mut Vec<ExclusiveShared<Tracker>>,
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

    /// Collects diagnostics on the resolving process.
    fn collect_diagnostics(
        &self,
        trackers_scores: &Vec<Vec<RecordScore>>,
        buckets: &Vec<ScoreBucket>,
    ) -> ResolvingDiagnostics {
        let mut diagnostics = ResolvingDiagnostics::new();

        let mut histogram_record_matchs = Histogram::new();
        let mut histogram_tracker_matchs = Histogram::new();

        for tracker_scores in trackers_scores.iter() {
            histogram_tracker_matchs.add(tracker_scores.len());
        }
        for bucket in buckets.iter() {
            histogram_record_matchs.add(bucket.scores().len());
        }

        diagnostics.histogram_record_matchs = histogram_record_matchs.into_vec();
        diagnostics.histogram_tracker_matchs = histogram_tracker_matchs.into_vec();

        diagnostics
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
        tracker_config: InternalTrackerConfig,
        trackers: &mut Vec<ExclusiveShared<Tracker>>,
        trackers_scores: Vec<Vec<RecordScore>>,
    ) -> (Vec<Tracker>, ResolvingDiagnostics) {
        let mut buckets = (0..frame.num_records())
            .into_iter()
            .map(|_| ScoreBucket::new())
            .collect::<Vec<ScoreBucket>>();

        for (tracker_scores, tracker) in trackers_scores.iter().zip(trackers.iter()) {
            for score in tracker_scores.iter() {
                buckets[score.idx].push(score.score, tracker.id());
            }
        }

        let diagnostics = self.collect_diagnostics(&trackers_scores, &buckets);

        let new_trackers = self.resolving_strategy.resolve(
            frame,
            tracker_config,
            trackers,
            buckets,
            trackers_scores,
        );

        (new_trackers, diagnostics)
    }
}
