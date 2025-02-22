use std::{collections::HashMap, hash::Hash};

/// DistanceMatrix
///
/// This is a building block for a cache, it stores the distances between elements.
/// To work properly, the distance between element must be symmetric, that is dist(a, b) == dist(b, a).
pub struct DistanceMatrix<K: Hash> {
    values: HashMap<(K, K), f32>,
}

impl<K: Hash + Eq + Ord> DistanceMatrix<K> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Clears the matrix.
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Returns the number of elements in the matrix.
    pub fn size(&self) -> usize {
        self.values.len()
    }

    /// Sets the distance between v1 and v2.
    pub fn set(&mut self, mut v1: K, mut v2: K, dist: f32) {
        if v2 < v1 {
            (v1, v2) = (v2, v1);
        }
        self.values.insert((v1, v2), dist);
    }

    /// Returns the distance between v1 and v2.
    pub fn get(&self, mut v1: K, mut v2: K) -> Option<f32> {
        if v2 < v1 {
            (v1, v2) = (v2, v1);
        }
        self.values.get(&(v1, v2)).copied()
    }
}
