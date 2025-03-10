use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{distances::compute_median_word, frame::Element, word::Word};

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
}
