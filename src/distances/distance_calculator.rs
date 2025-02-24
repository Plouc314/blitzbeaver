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

/// A cached distance calculator for elements
///
/// This is a wrapper around the type-specific cached distance calculators.
pub enum CachedDistanceCalculator {
    Word(CachedDistanceCalculatorWord),
    MultiWord(),
}

impl CachedDistanceCalculator {
    /// Returns the distance between two elements, either from the cache or by computing it.
    ///
    /// Note: this doesn't update the cache.
    pub fn get_dist(&mut self, e1: &Element, e2: &Element) -> f32 {
        match self {
            Self::Word(calculator) => match (e1, e2) {
                (Element::Word(w1), Element::Word(w2)) => calculator.get_dist(w1, w2),
                _ => 0.0,
            },
            Self::MultiWord() => {
                unimplemented!()
            }
        }
    }

    /// Clears the cache
    pub fn clear_cache(&mut self) {
        match self {
            Self::Word(calculator) => calculator.clear_cache(),
            Self::MultiWord() => {
                unimplemented!()
            }
        }
    }

    /// Pre-computes the distance between the most frequent uniques values to build the cache.
    pub fn precompute(&mut self, column1: &Vec<Element>, column2: &Vec<Element>) {
        match self {
            Self::Word(calculator) => calculator.precompute(column1, column2),
            Self::MultiWord() => {
                unimplemented!()
            }
        }
    }
}

/// A cached distance calculator for words
///
/// It builds a cache from the most frequent uniques values before the actual computation
/// to maximize the cache hit rate. The cache is immutable during the computation of a frame.
///
/// The cache should always be cleared after the computation of a frame.
pub struct CachedDistanceCalculatorWord {
    matrix: DistanceMatrix,
    distance_metric: Box<dyn DistanceMetric<Word>>,
    cache_dist_threshold: u32,
    #[cfg(feature = "benchmark")]
    pub trace: TraceCachedDistanceCalculator,
}

impl CachedDistanceCalculatorWord {
    pub fn new(distance: Box<dyn DistanceMetric<Word>>, cache_dist_threshold: u32) -> Self {
        Self {
            matrix: DistanceMatrix::new(),
            distance_metric: distance,
            cache_dist_threshold,
            #[cfg(feature = "benchmark")]
            trace: TraceCachedDistanceCalculator::new(),
        }
    }

    /// Returns the distance between two words, either from the cache or by computing it.
    ///
    /// Note: this doesn't update the cache.
    pub fn get_dist(&mut self, v1: &Word, v2: &Word) -> f32 {
        #[cfg(feature = "benchmark")]
        {
            self.trace.computation_count += 1;
        }

        match self.matrix.get(&v1.raw, &v2.raw) {
            Some(dist) => {
                #[cfg(feature = "benchmark")]
                {
                    self.trace.cache_hit_count += 1;
                }
                dist
            }
            None => self.distance_metric.dist(v1, v2),
        }
    }

    /// Clears the cache
    pub fn clear_cache(&mut self) {
        self.matrix.clear();
    }

    /// Computes the count of each unique word in the serie.
    fn compute_uniques<'a>(&self, serie: &'a Vec<Element>) -> HashMap<&'a Word, u32> {
        let mut uniques = HashMap::new();
        for e in serie.iter() {
            if let Element::Word(w) = &e {
                uniques.entry(w).and_modify(|c| *c += 1).or_insert(1);
            }
        }
        uniques
    }

    /// Pre-computes the distance between the most frequent uniques values to build the cache.
    pub fn precompute(&mut self, serie1: &Vec<Element>, serie2: &Vec<Element>) {
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
