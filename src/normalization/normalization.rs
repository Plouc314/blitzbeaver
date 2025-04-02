use crate::{
    distances::{compute_median_word, CachedDistanceCalculator},
    word::Word,
};

use super::clustering;

pub struct InternalNormalizationConfig {
    pub threshold_cluster_match: f32,
    pub min_cluster_size: usize,
}

pub struct Normalizer {
    config: InternalNormalizationConfig,
    distance_calculator: CachedDistanceCalculator,
}

impl Normalizer {
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
    fn build_clusters(&mut self, words: Vec<Option<&Word>>) -> (Vec<Word>, Vec<Option<usize>>) {
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
        let mut medians = Vec::new();
        let mut cluster_map = vec![None; words.len()];
        for (cluster_idx, cluster_set) in clusters_sets.iter().enumerate() {
            if cluster_set.len() < self.config.min_cluster_size {
                continue;
            }
            let mut cluster_words = Vec::new();
            for i in cluster_set.iter() {
                let idx = map_idx[i];
                cluster_map[idx] = Some(cluster_idx);
                cluster_words.push(words[idx].unwrap());
            }

            let median = compute_median_word(&cluster_words).unwrap();
            medians.push(median);
        }

        (medians, cluster_map)
    }

    fn get_right_cluster(&self, cluster_map: &Vec<Option<usize>>, mut idx: usize) -> Option<usize> {
        idx += 1;
        while idx < cluster_map.len() {
            if let Some(cluster) = cluster_map[idx] {
                return Some(cluster);
            }
        }
        None
    }

    fn infer_missing_clusters(&self, cluster_map: Vec<Option<usize>>) -> Vec<usize> {
        let mut cluster_map_inferred = vec![0; cluster_map.len()];
        let mut left_cluster = None;

        for i in 0..cluster_map.len() {
            if let Some(cluster) = cluster_map[i] {
                left_cluster = Some(cluster);
                cluster_map_inferred[i] = cluster;
                continue;
            }
            if let Some(cluster) = left_cluster {
                cluster_map_inferred[i] = cluster;
                continue;
            }
            if let Some(cluster) = self.get_right_cluster(&cluster_map, i) {
                cluster_map_inferred[i] = cluster;
                continue;
            }
        }

        cluster_map_inferred
    }

    pub fn normalize_words(&mut self, words: Vec<Option<&Word>>) -> Vec<Word> {
        let (medians, cluster_map) = self.build_clusters(words);
        let cluster_map = self.infer_missing_clusters(cluster_map);
        cluster_map
            .into_iter()
            .map(|idx| medians[idx].clone())
            .collect()
    }
}
