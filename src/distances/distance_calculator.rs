use std::collections::HashMap;

use crate::{
    distances::{Distance, DistanceMatrix},
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

pub enum CachedDistanceCalculator<'a> {
    Word(CachedDistanceCalculatorWord<'a>),
    MultiWord(),
}

impl<'a> CachedDistanceCalculator<'a> {
    pub fn get_dist(&mut self, e1: &Element<'a>, e2: &Element<'a>) -> f32 {
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

    pub fn precompute(&mut self, column1: &Vec<Element<'a>>, column2: &Vec<Element<'a>>) {
        match self {
            Self::Word(calculator) => calculator.precompute(column1, column2),
            Self::MultiWord() => {
                unimplemented!()
            }
        }
    }
}

pub struct CachedDistanceCalculatorWord<'a> {
    matrix: DistanceMatrix<&'a str>,
    distance: Box<dyn Distance<Word<'a>>>,
    cache_dist_threshold: u32,
    #[cfg(feature = "benchmark")]
    pub trace: TraceCachedDistanceCalculator,
}

impl<'a> CachedDistanceCalculatorWord<'a> {
    pub fn new(distance: Box<dyn Distance<Word<'a>>>, cache_dist_threshold: u32) -> Self {
        Self {
            matrix: DistanceMatrix::new(),
            distance,
            cache_dist_threshold,
            #[cfg(feature = "benchmark")]
            trace: TraceCachedDistanceCalculator::new(),
        }
    }

    pub fn get_dist(&mut self, v1: &Word<'a>, v2: &Word<'a>) -> f32 {
        #[cfg(feature = "benchmark")]
        {
            self.trace.computation_count += 1;
        }

        match self.matrix.get(v1.raw, v2.raw) {
            Some(dist) => {
                #[cfg(feature = "benchmark")]
                {
                    self.trace.cache_hit_count += 1;
                }
                dist
            }
            None => self.distance.dist(v1, v2),
        }
    }

    pub fn clear_cache(&mut self) {
        self.matrix.clear();
    }

    fn compute_uniques(&self, serie: &Vec<Element<'a>>) -> HashMap<Word<'a>, u32> {
        let mut uniques = HashMap::new();
        for e in serie.iter() {
            if let Element::Word(w) = e.clone() {
                uniques.entry(w).and_modify(|c| *c += 1).or_insert(1);
            }
        }
        uniques
    }

    pub fn precompute(&mut self, serie1: &Vec<Element<'a>>, serie2: &Vec<Element<'a>>) {
        let uniques1 = self.compute_uniques(serie1);
        let uniques2 = self.compute_uniques(serie2);

        for (v1, c1) in uniques1.iter() {
            for (v2, c2) in uniques2.iter() {
                // only pre-compute and store when a min of occurence is reached
                if *c1 * *c2 < self.cache_dist_threshold {
                    continue;
                }

                if self.matrix.get(&v1.raw, &v2.raw).is_none() {
                    let dist = self.distance.dist(v1, v2);
                    self.matrix.set(&v1.raw, v2.raw, dist);
                }
            }
        }

        #[cfg(feature = "benchmark")]
        {
            self.trace.cache_size = self.matrix.size();
        }
    }
}
