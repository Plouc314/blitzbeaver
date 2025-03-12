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
    fn score(&self, scores: &Vec<Option<f32>>) -> f32 {
        scores.iter().filter_map(|s| *s).sum::<f32>() / scores.len() as f32
    }
}

/// WeightedAverageRecordScorer
///
/// Computes the weighted arithmetic mean of the scores.
pub struct WeightedAverageRecordScorer {
    weights: Vec<f32>,
    min_weight_ratio: f32,
}

impl WeightedAverageRecordScorer {
    pub fn new(weights: Vec<f32>, min_weight_ratio: f32) -> Self {
        Self {
            weights,
            min_weight_ratio,
        }
    }
}

impl RecordScorer for WeightedAverageRecordScorer {
    fn score(&self, scores: &Vec<Option<f32>>) -> f32 {
        let tot_weight = self.weights.iter().sum::<f32>();
        let mut tot_score = 0.0;
        let mut effective_weight = 0.0;
        for i in 0..self.weights.len() {
            if let Some(score) = &scores[i] {
                tot_score += score * self.weights[i];
                effective_weight += self.weights[i];
            }
        }

        tot_score / f32::max(effective_weight, self.min_weight_ratio * tot_weight)
    }
}

/// WeightedQuadraticRecordScorer
///
/// Computes the weighted quadratic mean of the scores.
pub struct WeightedQuadraticRecordScorer {
    weights: Vec<f32>,
    min_weight_ratio: f32,
}

impl WeightedQuadraticRecordScorer {
    pub fn new(weights: Vec<f32>, min_weight_ratio: f32) -> Self {
        Self {
            weights,
            min_weight_ratio,
        }
    }
}

impl RecordScorer for WeightedQuadraticRecordScorer {
    fn score(&self, scores: &Vec<Option<f32>>) -> f32 {
        let tot_weight = self.weights.iter().sum::<f32>();
        let mut tot_score = 0.0;
        let mut effective_weight = 0.0;
        for i in 0..self.weights.len() {
            if let Some(score) = &scores[i] {
                tot_score += score * score * self.weights[i];
                effective_weight += self.weights[i];
            }
        }

        tot_score / f32::max(effective_weight, self.min_weight_ratio * tot_weight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_eq_f32(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-6);
    }

    #[test]
    fn test_average_scorer() {
        let scorer = AverageRecordScorer::new();

        // check average on normal scores
        let scores = vec![Some(0.5), Some(0.6), Some(0.7)];
        assert_eq_f32(scorer.score(&scores), 0.6);

        // check average with some missing scores
        let scores = vec![Some(0.5), None, Some(0.7)];
        assert_eq_f32(scorer.score(&scores), 0.3999999);

        // check average with all missing scores
        let scores = vec![None, None, None];
        assert_eq_f32(scorer.score(&scores), 0.0);
    }

    #[test]
    fn test_weighted_average_scorer() {
        let weights = vec![1.0, 2.0, 3.0];
        let min_weight_ratio = 0.5;
        let scorer = WeightedAverageRecordScorer::new(weights, min_weight_ratio);

        // check weighted average on normal scores
        let scores = vec![Some(0.5), Some(0.6), Some(0.7)];
        assert_eq_f32(scorer.score(&scores), 0.6333333);

        // check weighted average with some missing scores
        // the ratio of effective weight is of 4/6 which is higher than the min_weight_ratio
        // so the weight utilized is (1.0 + 3.0)
        let scores = vec![Some(0.5), None, Some(0.7)];
        assert_eq_f32(scorer.score(&scores), 0.6499999);

        // check weighted average with lot of missing scores
        // the ratio of effective weight is of 2/6 which is lower than the min_weight_ratio
        // so the weight utilized is (1.0 + 2.0 + 3.0) * min_weight_ratio
        let scores = vec![None, Some(0.6), None];
        assert_eq_f32(scorer.score(&scores), 0.3999999);

        // check weighted average with all missing scores
        let scores = vec![None, None, None];
        assert_eq_f32(scorer.score(&scores), 0.0);
    }

    #[test]
    fn test_weighted_quadratic_scorer() {
        let weights = vec![1.0, 2.0, 3.0];
        let min_weight_ratio = 0.5;
        let scorer = WeightedQuadraticRecordScorer::new(weights, min_weight_ratio);

        // check weighted quadratic on normal scores
        let scores = vec![Some(0.5), Some(0.6), Some(0.7)];
        assert_eq_f32(scorer.score(&scores), 0.4066666);

        // check weighted quadratic with some missing scores
        // the ratio of effective weight is of 4/6 which is higher than the min_weight_ratio
        // so the weight utilized is (1.0 + 3.0)
        let scores = vec![Some(0.5), None, Some(0.7)];
        assert_eq_f32(scorer.score(&scores), 0.4299999);

        // check weighted quadratic with lot of missing scores
        // the ratio of effective weight is of 2/6 which is lower than the min_weight_ratio
        // so the weight utilized is (1.0 + 2.0 + 3.0) * min_weight_ratio
        let scores = vec![None, Some(0.6), None];
        assert_eq_f32(scorer.score(&scores), 0.24);

        // check weighted quadratic with all missing scores
        let scores = vec![None, None, None];
        assert_eq_f32(scorer.score(&scores), 0.0);
    }
}
