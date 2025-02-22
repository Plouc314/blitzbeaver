use crate::{
    frame::Frame,
    id::ID,
    trackers::{RecordScore, Tracker},
};

pub struct ScoreBucket {
    scores: Vec<(f32, ID)>,
}

impl ScoreBucket {
    pub fn new() -> Self {
        Self { scores: Vec::new() }
    }

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

pub trait ResolvingStrategy<'a> {
    fn resolve(
        &mut self,
        frame: &Frame<'a>,
        trackers: &mut Vec<Box<dyn Tracker<'a>>>,
        buckets: Vec<ScoreBucket>,
        trackers_scores: Vec<Vec<RecordScore>>,
    ) -> Vec<Box<dyn Tracker<'a>>>;
}

pub struct Resolver<'a> {
    resolving_strategy: Box<dyn ResolvingStrategy<'a>>,
}

impl<'a> Resolver<'a> {
    pub fn new(resolving_strategy: Box<dyn ResolvingStrategy<'a>>) -> Self {
        Self { resolving_strategy }
    }

    pub fn resolve(
        &mut self,
        frame: &Frame<'a>,
        trackers: &mut Vec<Box<dyn Tracker<'a>>>,
        trackers_scores: Vec<Vec<RecordScore>>,
    ) -> Vec<Box<dyn Tracker<'a>>> {
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
