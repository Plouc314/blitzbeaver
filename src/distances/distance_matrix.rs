use std::{collections::HashMap, hash::Hash};

pub struct DistanceMatrix<K: Hash> {
    values: HashMap<(K, K), f32>,
}

impl<K: Hash + Eq + Ord> DistanceMatrix<K> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn size(&self) -> usize {
        self.values.len()
    }

    pub fn set(&mut self, mut v1: K, mut v2: K, dist: f32) {
        if v2 < v1 {
            (v1, v2) = (v2, v1);
        }
        self.values.insert((v1, v2), dist);
    }

    pub fn get(&self, mut v1: K, mut v2: K) -> Option<f32> {
        if v2 < v1 {
            (v1, v2) = (v2, v1);
        }
        self.values.get(&(v1, v2)).copied()
    }
}
