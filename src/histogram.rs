/// Histogram
///
/// Implemented using a vector of buckets, as such values
/// should be densely distributed.
pub struct Histogram {
    buckets: Vec<usize>,
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            buckets: Vec::new(),
        }
    }

    /// Adds a value to the histogram.
    ///
    /// This will resize the histogram if needed.
    pub fn add(&mut self, value: usize) {
        if value >= self.buckets.len() {
            self.buckets.resize(value + 1, 0);
        }
        self.buckets[value] += 1;
    }

    /// Converts the histogram into a vector.
    pub fn into_vec(self) -> Vec<usize> {
        self.buckets
    }
}
