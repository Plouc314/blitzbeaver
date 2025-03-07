use super::tracker::RecordScorer;

/// AverageRecordScorer
///
/// Computes the arithmetic mean of the scores, regardless of the columns.
pub struct AverageRecordScorer {}

impl AverageRecordScorer {
    pub fn new() -> Self {
        Self {}
    }
}

impl RecordScorer for AverageRecordScorer {
    fn score(&self, scores: &Vec<f32>) -> f32 {
        scores.iter().sum::<f32>() / scores.len() as f32
    }
}

/// WeightedAverageRecordScorer
///
/// Computes the weighted arithmetic mean of the scores.
pub struct WeightedAverageRecordScorer {
    weights: Vec<f32>,
}

impl WeightedAverageRecordScorer {
    pub fn new(weights: Vec<f32>) -> Self {
        Self { weights }
    }
}

impl RecordScorer for WeightedAverageRecordScorer {
    fn score(&self, scores: &Vec<f32>) -> f32 {
        scores
            .iter()
            .zip(self.weights.iter())
            .map(|(score, weight)| score * weight)
            .sum::<f32>()
            / self.weights.iter().sum::<f32>()
    }
}

/// WeightedQuadraticRecordScorer
///
/// Computes the weighted quadratic mean of the scores.
pub struct WeightedQuadraticRecordScorer {
    weights: Vec<f32>,
}

impl WeightedQuadraticRecordScorer {
    pub fn new(weights: Vec<f32>) -> Self {
        Self { weights }
    }
}

impl RecordScorer for WeightedQuadraticRecordScorer {
    fn score(&self, scores: &Vec<f32>) -> f32 {
        scores
            .iter()
            .zip(self.weights.iter())
            .map(|(score, weight)| score * score * weight)
            .sum::<f32>()
            / self.weights.iter().sum::<f32>()
    }
}
