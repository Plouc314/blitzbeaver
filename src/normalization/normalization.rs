use crate::{
    distances::{compute_median_word, CachedDistanceCalculator, InternalDistanceMetricConfig},
    word::Word,
};

use super::clustering;

pub struct InternalNormalizationConfig {
    pub threshold_cluster_match: f32,
    pub min_cluster_size: usize,
    pub infer_missing_clusters: bool,
}

/// Normalizer
///
/// Responsible for normalizing words and multi-words by clustering them
/// and replacing each word with the median of its cluster.
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

    /// Builds clusters of words based on their distances and computes the median word for each cluster.
    /// Returns a vector of median words and a mapping of original words to their respective cluster indices.
    /// The mapping is `None` for words that are not part of any cluster.
    fn build_clusters(&mut self, words: &Vec<Option<&Word>>) -> (Vec<Word>, Vec<Option<usize>>) {
        let mut map_idx = Vec::with_capacity(words.len());
        let mut non_null_words = Vec::with_capacity(words.len());
        for (i, word) in words.iter().enumerate() {
            if let Some(word) = word {
                non_null_words.push(*word);
                map_idx.push(i);
            }
        }
        let mut clusters_sets = clustering::compute_words_clusters(
            &mut self.distance_calculator,
            non_null_words,
            self.config.threshold_cluster_match,
        );
        clusters_sets = clusters_sets
            .into_iter()
            .filter(|c| c.len() >= self.config.min_cluster_size)
            .collect();

        let mut medians = Vec::new();
        let mut cluster_map = vec![None; words.len()];
        for (cluster_idx, cluster_set) in clusters_sets.iter().enumerate() {
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
            idx += 1;
        }
        None
    }

    /// Infers clusters for words that are not part of any cluster by checking the nearest cluster to the left or right.
    ///
    /// This is a simple heuristic, may be subject to refinement.
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

    /// Normalizes a vector of "single" words by clustering them and replacing each word with the median of its cluster.
    ///
    /// This approach assign a single cluster to each frame.
    ///
    /// In case no clusters are found, the original words are returned.
    pub fn normalize_words(&mut self, words: Vec<Option<&Word>>) -> Vec<Option<Word>> {
        let (medians, cluster_map) = self.build_clusters(&words);

        if medians.len() == 0 {
            return words.into_iter().map(|w| w.map(|w| w.clone())).collect();
        }

        if self.config.infer_missing_clusters {
            let cluster_map = self.infer_missing_clusters(cluster_map);
            cluster_map
                .into_iter()
                .map(|idx| Some(medians[idx].clone()))
                .collect()
        } else {
            cluster_map
                .into_iter()
                .enumerate()
                .map(|(i, cluster_idx)| match cluster_idx {
                    Some(cluster_idx) => Some(medians[cluster_idx].clone()),
                    None => words[i].cloned(),
                })
                .collect()
        }
    }

    /// Normalizes a vector of "multi-words" by clustering them and replacing each word with the median of its cluster.
    ///
    /// Attributes a range to each cluster, filling any missing word in a frame with the median of the cluster.
    pub fn normalize_multi_words(&mut self, words: Vec<&Vec<Word>>) -> Vec<Vec<Word>> {
        let mut map_flat_word_frame_idx = Vec::with_capacity(words.len());
        let mut flat_words = Vec::new();
        for (frame_idx, frame) in words.iter().enumerate() {
            for word in frame.iter() {
                flat_words.push(Some(word));
                map_flat_word_frame_idx.push(frame_idx);
            }
        }
        let (medians, cluster_map) = self.build_clusters(&flat_words);
        let mut clusters_range = vec![(usize::MAX, 0); medians.len()];
        for (i, cluster) in cluster_map.iter().enumerate() {
            if let Some(cluster) = cluster {
                let frame_idx = map_flat_word_frame_idx[i];
                if clusters_range[*cluster].0 > frame_idx {
                    clusters_range[*cluster].0 = frame_idx;
                }
                if clusters_range[*cluster].1 < frame_idx {
                    clusters_range[*cluster].1 = frame_idx;
                }
            }
        }
        let mut inferred_words = vec![Vec::new(); words.len()];
        for (cluster_idx, (start, end)) in clusters_range.into_iter().enumerate() {
            for i in start..=end {
                inferred_words[i].push(medians[cluster_idx].clone());
            }
        }
        inferred_words
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{distances::LvOptiDistanceMetric, word::Word};

    fn make_normalizer(config: InternalNormalizationConfig) -> Normalizer {
        Normalizer::new(
            config,
            CachedDistanceCalculator::new(Box::new(LvOptiDistanceMetric::new(false)), 4),
        )
    }

    #[test]
    fn test_normalize_words() {
        let words = vec![
            Some(Word::new("magimelien".to_string())),
            Some(Word::new("mazimilien".to_string())),
            Some(Word::new("mazirelien".to_string())),
            Some(Word::new("marinelien".to_string())),
            Some(Word::new("hgdfzs".to_string())),
            Some(Word::new("bob".to_string())),
            Some(Word::new("boob".to_string())),
        ];
        let words = words.iter().map(|w| w.as_ref()).collect();
        let mut normalizer = make_normalizer(InternalNormalizationConfig {
            threshold_cluster_match: 0.6,
            min_cluster_size: 2,
            infer_missing_clusters: true,
        });
        let normalized_words = normalizer.normalize_words(words);
        assert_eq!(
            normalized_words,
            vec![
                Some(Word::new("mazimelien".to_string())),
                Some(Word::new("mazimelien".to_string())),
                Some(Word::new("mazimelien".to_string())),
                Some(Word::new("mazimelien".to_string())),
                Some(Word::new("mazimelien".to_string())),
                Some(Word::new("bob".to_string())),
                Some(Word::new("bob".to_string())),
            ]
        );
    }

    #[test]
    fn test_normalize_multi_words() {
        let words = vec![
            vec![
                Word::new("emma".to_string()),
                Word::new("edouard".to_string()),
            ],
            vec![
                Word::new("gustave".to_string()),
                Word::new("edouard".to_string()),
            ],
            vec![
                Word::new("emma".to_string()),
                Word::new("doriard".to_string()),
            ],
            vec![
                Word::new("emma".to_string()),
                Word::new("gabriel".to_string()),
            ],
            vec![
                Word::new("auden".to_string()),
                Word::new("emma".to_string()),
            ],
            vec![
                Word::new("emma".to_string()),
                Word::new("edinard".to_string()),
            ],
            vec![
                Word::new("emma".to_string()),
                Word::new("edouard".to_string()),
            ],
            vec![Word::new("edouard".to_string())],
            vec![Word::new("edouard".to_string())],
        ];
        let mut normalizer = make_normalizer(InternalNormalizationConfig {
            threshold_cluster_match: 0.6,
            min_cluster_size: 2,
            infer_missing_clusters: true,
        });
        let normalized_words = normalizer.normalize_multi_words(words.iter().collect());
        assert_eq!(
            normalized_words,
            vec![
                vec![
                    Word::new("emma".to_string()),
                    Word::new("edouard".to_string()),
                ],
                vec![
                    Word::new("emma".to_string()),
                    Word::new("edouard".to_string()),
                ],
                vec![
                    Word::new("emma".to_string()),
                    Word::new("edouard".to_string()),
                ],
                vec![
                    Word::new("emma".to_string()),
                    Word::new("edouard".to_string()),
                ],
                vec![
                    Word::new("emma".to_string()),
                    Word::new("edouard".to_string()),
                ],
                vec![
                    Word::new("emma".to_string()),
                    Word::new("edouard".to_string()),
                ],
                vec![
                    Word::new("emma".to_string()),
                    Word::new("edouard".to_string()),
                ],
                vec![Word::new("edouard".to_string())],
                vec![Word::new("edouard".to_string())],
            ]
        )
    }
}
