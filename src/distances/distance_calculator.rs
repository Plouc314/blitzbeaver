use std::collections::HashMap;

use crate::{
    distances::{DistanceMatrix, DistanceMetric},
    frame::Element,
    word::Word,
};

#[cfg(feature = "benchmark")]
#[derive(Clone)]
pub struct TraceCachedDistanceCalculator {
    pub computation_count: u64,
    pub cache_hit_count: u64,
    pub cache_size: usize,
}

#[cfg(feature = "benchmark")]
impl TraceCachedDistanceCalculator {
    pub fn new() -> Self {
        Self {
            computation_count: 0,
            cache_hit_count: 0,
            cache_size: 0,
        }
    }

    pub fn reset(&mut self) {
        self.computation_count = 0;
        self.cache_hit_count = 0;
        self.cache_size = 0;
    }

    pub fn merge(&mut self, other: Self) {
        self.computation_count += other.computation_count;
        self.cache_hit_count += other.cache_hit_count;
        self.cache_size += other.cache_size;
    }
}

/// A cached distance calculator
///
/// It builds a cache from the most frequent uniques values before the actual computation
/// to maximize the cache hit rate. The cache is immutable during the computation of a frame.
///
/// The cache should always be cleared after the computation of a frame.
pub struct CachedDistanceCalculator {
    matrix: DistanceMatrix,
    distance_metric: Box<dyn DistanceMetric<Word> + Send>,
    cache_dist_threshold: u32,
    #[cfg(feature = "benchmark")]
    pub trace: TraceCachedDistanceCalculator,
}

impl CachedDistanceCalculator {
    pub fn new(distance: Box<dyn DistanceMetric<Word> + Send>, cache_dist_threshold: u32) -> Self {
        Self {
            matrix: DistanceMatrix::new(),
            distance_metric: distance,
            cache_dist_threshold,
            #[cfg(feature = "benchmark")]
            trace: TraceCachedDistanceCalculator::new(),
        }
    }

    pub fn get_dist(&mut self, e1: &Element, e2: &Element) -> Option<f32> {
        match (e1, e2) {
            (Element::Word(w1), Element::Word(w2)) => Some(self.get_dist_word(w1, w2)),
            (Element::MultiWords(ws1), Element::MultiWords(ws2)) => self.get_dists_words(ws1, ws2),
            _ => None,
        }
    }

    /// Returns the distance between two words, either from the cache or by computing it.
    ///
    /// Note: this doesn't update the cache.
    pub fn get_dist_word(&mut self, w1: &Word, w2: &Word) -> f32 {
        #[cfg(feature = "benchmark")]
        {
            self.trace.computation_count += 1;
        }

        match self.matrix.get(&w1.raw, &w2.raw) {
            Some(dist) => {
                #[cfg(feature = "benchmark")]
                {
                    self.trace.cache_hit_count += 1;
                }
                dist
            }
            None => self.distance_metric.dist(w1, w2),
        }
    }

    /// Computes the distance between two vectors of words.
    ///
    /// Compares from the perspective of the first vector to the second one,
    /// for each word in the first vector, computes the distance to all words in the second vector
    /// and keeps the maximum distance.
    ///
    /// Returns the average distance or None if the first vector is empty.
    pub fn get_dists_words(&mut self, ws1: &Vec<Word>, ws2: &Vec<Word>) -> Option<f32> {
        if ws1.len() == 0 {
            return None;
        }
        let mut tot_dist = 0.0;
        for w1 in ws1.iter() {
            // the dist is 0 if ws2 is empty
            let dist = ws2
                .iter()
                .map(|w2| self.get_dist_word(w1, w2))
                .reduce(f32::max)
                .unwrap_or(0.0);
            tot_dist += dist;
        }

        let max_len = usize::max(ws1.len(), ws2.len());
        let agg_dist = tot_dist / max_len as f32;
        Some(agg_dist)
    }

    /// Clears the cache
    pub fn clear_cache(&mut self) {
        self.matrix.clear();
    }

    /// Returns the size of the cache.
    pub fn cache_size(&self) -> usize {
        self.matrix.size()
    }

    /// Computes the count of each unique word in the serie.
    fn compute_uniques<'a>(&self, serie: &'a Vec<&'a Element>) -> HashMap<&'a Word, u32> {
        let mut uniques = HashMap::new();
        for e in serie.iter() {
            if let Element::Word(w) = e {
                uniques.entry(w).and_modify(|c| *c += 1).or_insert(1);
            }
        }
        uniques
    }

    /// Pre-computes the distance between the most frequent uniques values to build the cache.
    pub fn precompute(&mut self, serie1: &Vec<&Element>, serie2: &Vec<&Element>) {
        let uniques1 = self.compute_uniques(serie1);
        let uniques2 = self.compute_uniques(serie2);

        for (v1, c1) in uniques1.iter() {
            for (v2, c2) in uniques2.iter() {
                // only pre-compute and store when a min of occurence is reached
                if *c1 * *c2 < self.cache_dist_threshold {
                    continue;
                }

                if self.matrix.get(&v1.raw, &v2.raw).is_none() {
                    let dist = self.distance_metric.dist(v1, v2);
                    self.matrix.set(&v1.raw, &v2.raw, dist);
                }
            }
        }

        #[cfg(feature = "benchmark")]
        {
            self.trace.cache_size = self.matrix.size();
        }
    }
}

impl Clone for CachedDistanceCalculator {
    fn clone(&self) -> Self {
        Self {
            matrix: self.matrix.clone(),
            distance_metric: self.distance_metric.clone(),
            cache_dist_threshold: self.cache_dist_threshold,
            #[cfg(feature = "benchmark")]
            trace: self.trace.clone(),
        }
    }
}
