use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

/// DistanceMatrix
///
/// This is a building block for a cache, it stores the distances between elements.
/// To work properly, the distance between element must be symmetric, that is dist(a, b) == dist(b, a).
///
/// Note: the distance matrix computes the hashes of the keys passed to avoid having a reference
/// as key, to avoid lifetime complications.
#[derive(Clone)]
pub struct DistanceMatrix {
    values: HashMap<u64, f32>,
}

impl DistanceMatrix {
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

    /// Hashes two keys and returns the hash.
    ///
    /// Keys are ordered before hashing, such that hash(a, b) == hash(b, a)
    fn hash<K: Hash + Eq + Ord>(v1: &K, v2: &K) -> u64 {
        let mut hasher = DefaultHasher::new();
        if v2 < v1 {
            v2.hash(&mut hasher);
            v1.hash(&mut hasher);
        } else {
            v1.hash(&mut hasher);
            v2.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Sets the distance between v1 and v2.
    pub fn set<K: Hash + Eq + Ord>(&mut self, v1: &K, v2: &K, dist: f32) {
        self.values.insert(Self::hash(v1, v2), dist);
    }

    /// Returns the distance between v1 and v2.
    pub fn get<K: Hash + Eq + Ord>(&self, v1: &K, v2: &K) -> Option<f32> {
        self.values.get(&Self::hash(v1, v2)).copied()
    }
}
