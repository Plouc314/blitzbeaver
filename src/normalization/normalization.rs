use crate::{distances::CachedDistanceCalculator, word::Word};

use super::clustering;

pub struct InternalNormalizationConfig {
    pub threshold_cluster_match: f32,
    pub min_cluster_size: usize,
}

pub struct Normalization {
    config: InternalNormalizationConfig,
    distance_calculator: CachedDistanceCalculator,
}

impl Normalization {
    pub fn new(
        config: InternalNormalizationConfig,
        distance_calculator: CachedDistanceCalculator,
    ) -> Self {
        Self {
            config,
            distance_calculator,
        }
    }

    /// Builds a vector mapping each word to its cluster index or None if it is not part of a cluster.
    fn build_cluster_map(&mut self, words: Vec<Option<&Word>>) -> Vec<Option<usize>> {
        let mut map_idx = Vec::with_capacity(words.len());
        let mut non_null_words = Vec::with_capacity(words.len());
        for (i, word) in words.iter().enumerate() {
            if let Some(word) = word {
                non_null_words.push(*word);
                map_idx.push(i);
            }
        }
        let clusters_sets = clustering::compute_words_clusters(
            &mut self.distance_calculator,
            non_null_words,
            self.config.threshold_cluster_match,
        );
        let mut cluster_map = vec![None; words.len()];
        for (cluster_idx, cluster_set) in clusters_sets.into_iter().enumerate() {
            if cluster_set.len() < self.config.min_cluster_size {
                continue;
            }
            for i in cluster_set.iter() {
                let idx = map_idx[i];
                cluster_map[idx] = Some(cluster_idx);
            }
        }
        cluster_map
    }

    fn infer_missing_clusters(&self, cluster_map: Vec<Option<usize>>) -> Vec<usize> {
        let mut left_cluster = None;
        let mut cluster_map_inferred = vec![0; cluster_map.len()];
    }

    pub fn normalize_words(&mut self, words: Vec<Option<&Word>>) {
        let cluster_map = self.build_cluster_map(words);
    }
}
