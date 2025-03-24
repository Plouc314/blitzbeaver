use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
    distances::{compute_median_word, DistanceMetric},
    frame::Element,
    word::Word,
};

use super::tracker::TrackerMemory;

/// BruteForceMemory
///
/// Always returns all the elements that have been seen.
pub struct BruteForceMemory {
    elements: Vec<Element>,
}

impl BruteForceMemory {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
}

impl TrackerMemory for BruteForceMemory {
    fn signal_no_matching_element(&mut self) {}

    fn signal_matching_element(&mut self, element: Element) {
        if element.is_none() {
            return;
        }
        self.elements.push(element);
    }

    fn get_elements(&self) -> Vec<&Element> {
        self.elements.iter().map(|e| e).collect()
    }

    fn new_default(&self) -> Box<dyn TrackerMemory + Send + Sync> {
        Box::new(Self::new())
    }
}

/// MostFrequentMemory
///
/// Returns the most frequent element that has been seen.
pub struct MostFrequentMemory {
    mf_count: u32,
    elements: Vec<Element>,
    mf_indexes: Vec<usize>,
    unique_counts: HashMap<u64, u32>,
}

impl MostFrequentMemory {
    pub fn new() -> Self {
        Self {
            mf_count: 0,
            elements: Vec::new(),
            mf_indexes: Vec::new(),
            unique_counts: HashMap::new(),
        }
    }

    fn hash_element(element: &Element) -> u64 {
        let mut hasher = DefaultHasher::new();
        element.hash(&mut hasher);
        hasher.finish()
    }
}

impl TrackerMemory for MostFrequentMemory {
    fn signal_no_matching_element(&mut self) {}

    fn signal_matching_element(&mut self, element: Element) {
        if element.is_none() {
            return;
        }

        let hash = Self::hash_element(&element);
        let idx = self.elements.len();
        self.elements.push(element);

        let mut count = 1;
        self.unique_counts
            .entry(hash)
            .and_modify(|v| {
                *v += 1;
                count = *v;
            })
            .or_insert(1);

        if count > self.mf_count {
            self.mf_count = count;
            self.mf_indexes.clear();
            self.mf_indexes.push(idx);
        } else if count == self.mf_count {
            self.mf_indexes.push(idx);
        }
    }

    fn get_elements(&self) -> Vec<&Element> {
        self.mf_indexes
            .iter()
            .map(|idx| &self.elements[*idx])
            .collect()
    }

    fn new_default(&self) -> Box<dyn TrackerMemory + Send + Sync> {
        Box::new(Self::new())
    }
}

/// LongShortTermMemory
///
/// Composed with another memory, it returns:
/// - Long term memory: returns the elements of the composed memory.
/// - Short term memory: returns the latest element that has been seen.
pub struct LongShortTermMemory {
    long_memory: Box<dyn TrackerMemory + Send + Sync>,
    latest_element: Option<Element>,
}

impl LongShortTermMemory {
    pub fn new(long_memory: Box<dyn TrackerMemory + Send + Sync>) -> Self {
        Self {
            long_memory,
            latest_element: None,
        }
    }
}

impl TrackerMemory for LongShortTermMemory {
    fn signal_no_matching_element(&mut self) {
        self.long_memory.signal_no_matching_element();
    }

    fn signal_matching_element(&mut self, element: Element) {
        if element.is_none() {
            return;
        }

        let mut element = Some(element);
        std::mem::swap(&mut self.latest_element, &mut element);
        if let Some(element) = element {
            self.long_memory.signal_matching_element(element);
        }
    }

    fn get_elements(&self) -> Vec<&Element> {
        let mut elements = self.long_memory.get_elements();
        if let Some(element) = &self.latest_element {
            elements.push(element);
        }
        elements
    }

    fn new_default(&self) -> Box<dyn TrackerMemory + Send + Sync> {
        Box::new(Self::new(self.long_memory.new_default()))
    }
}

/// MedianWordMemory
///
/// Computes and returns the median word from the words that have been seen.
pub struct MedianWordMemory {
    elements: Vec<Element>,
    median_word: Option<Element>,
}

impl MedianWordMemory {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            median_word: None,
        }
    }
}

impl TrackerMemory for MedianWordMemory {
    fn signal_no_matching_element(&mut self) {}

    fn signal_matching_element(&mut self, element: Element) {
        if element.is_none() {
            return;
        }
        self.elements.push(element);
        let median_word = compute_median_word(
            &self
                .elements
                .iter()
                .filter_map(|e| match e {
                    Element::Word(w) => Some(w),
                    _ => None,
                })
                .collect::<Vec<&Word>>(),
        );

        self.median_word = median_word.map(Element::Word);
    }

    fn get_elements(&self) -> Vec<&Element> {
        match self.median_word {
            Some(ref w) => vec![w],
            None => Vec::new(),
        }
    }

    fn new_default(&self) -> Box<dyn TrackerMemory + Send + Sync> {
        Box::new(Self::new())
    }
}

/// MultiWordMemory
///
/// Tracks dynamically an arbitrary number of words, each word
/// is associated with a memory that tracks the word.
///
/// For each word, it compares it with all the words of all the memories
/// and matches it with the memory that is most similar to it.
/// In case none of the memories is similar enough, it creates a new memory.
///
/// For simplicity of the implementation, the following is assumed:
/// - A word may match with at most one memory.
/// - The memory strategy used for the words "produces" a single element,
///   that is no brute force memory for example.
pub struct MultiWordMemory {
    memories: Vec<Box<dyn TrackerMemory + Send + Sync>>,
    template: Box<dyn TrackerMemory + Send + Sync>,
    distance_metric: Box<dyn DistanceMetric<Word> + Send + Sync>,
    threshold_match: f32,
    current_element: Option<Element>,
}

impl MultiWordMemory {
    pub fn new(
        memory: Box<dyn TrackerMemory + Send + Sync>,
        distance_metric: Box<dyn DistanceMetric<Word> + Send + Sync>,
        threshold_match: f32,
    ) -> Self {
        Self {
            memories: Vec::new(),
            template: memory,
            distance_metric,
            threshold_match,
            current_element: None,
        }
    }

    /// Compares the word with all the words of all the memories
    /// and returns the index of the memory and the distance of the word
    /// with the maximum distance.
    fn get_word_dist(&mut self, w1: &Word) -> (usize, f32) {
        let mut max_dist = 0.0;
        let mut max_idx = 0;
        for memory in &self.memories {
            for (idx, element) in memory.get_elements().iter().enumerate() {
                if let Element::Word(w2) = element {
                    let dist = self.distance_metric.dist(w1, w2);
                    if dist > max_dist {
                        max_dist = dist;
                        max_idx = idx;
                    }
                }
            }
        }
        (max_idx, max_dist)
    }

    /// note: assumes that the memories have a single element
    /// as otherwise it would be necessary to returns every
    /// permutations of the elements of the memories which
    /// would be terribly inefficient
    fn build_elements(&mut self) {
        let mut words = Vec::new();
        for memory in &self.memories {
            let elements = memory.get_elements();
            if let Some(Element::Word(w)) = elements.first() {
                words.push(w.clone());
            }
        }
        self.current_element = Some(Element::MultiWords(words));
    }
}

impl TrackerMemory for MultiWordMemory {
    fn signal_no_matching_element(&mut self) {}

    fn signal_matching_element(&mut self, element: Element) {
        let words = match element {
            Element::MultiWords(ws) => ws,
            _ => return,
        };

        // note: assumes that no two words match with the same memory
        for word in words.into_iter() {
            let (idx, dist) = self.get_word_dist(&word);
            if dist >= self.threshold_match {
                self.memories[idx].signal_matching_element(Element::Word(word));
            } else {
                let mut memory = self.template.new_default();
                memory.signal_matching_element(Element::Word(word.clone()));
                self.memories.push(memory);
            }
        }

        self.build_elements();
    }

    fn get_elements(&self) -> Vec<&Element> {
        match &self.current_element {
            Some(e) => vec![e],
            None => Vec::new(),
        }
    }

    fn new_default(&self) -> Box<dyn TrackerMemory + Send + Sync> {
        Box::new(Self::new(
            self.template.new_default(),
            self.distance_metric.clone(),
            self.threshold_match,
        ))
    }
}
