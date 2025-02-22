use crate::word::Word;

/// Represents an element within a `Record` or `Frame`.
///
/// An `Element` can be:
/// - A single `Word`
/// - A collection of multiple `Word`s
/// - `None`, indicating an absence of data.
#[derive(Clone)]
pub enum Element<'a> {
    Word(Word<'a>),
    MultiWords(Vec<Word<'a>>),
    None,
}

/// Represents a row in a `Frame`.
///
/// A `Record` is composed of multiple `Element`s, each corresponding to a column
/// in the `Frame`.
#[derive(Default, Clone)]
pub struct Record<'a> {
    elements: Vec<Element<'a>>,
}

impl<'a> Record<'a> {
    /// Creates a new `Record` from a vector of `Element`s.
    pub fn new(elements: Vec<Element<'a>>) -> Self {
        Self { elements }
    }

    /// Returns the number of elements (features) in the record.
    pub fn size(&self) -> usize {
        self.elements.len()
    }

    /// Returns a reference to the element at the specified index.
    ///
    /// # Arguments
    ///
    /// * `idx` - The index of the element to retrieve.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn element(&self, idx: usize) -> &Element<'a> {
        &self.elements[idx]
    }
}

/// Represents a structured collection of records.
///
/// The `Frame` stores data in a column-major format, meaning each column contains
/// all values for a specific feature across all records. This improves cache
/// locality when processing data column-wise.
pub struct Frame<'a> {
    idx: usize,
    columns: Vec<Vec<Element<'a>>>,
}

impl<'a> Frame<'a> {
    /// Creates a new `Frame` from a vector of columns, where each column
    /// contains elements for all records.
    pub fn new(idx: usize, columns: Vec<Vec<Element<'a>>>) -> Self {
        Self { idx, columns }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    /// Returns the number of records (rows) in the `Frame`.
    pub fn num_records(&self) -> usize {
        self.columns[0].len()
    }

    /// Returns the number of features (columns) in the `Frame`.
    pub fn num_features(&self) -> usize {
        self.columns.len()
    }

    /// Returns a reference to the column at the specified index.
    ///
    /// # Arguments
    ///
    /// * `idx` - The index of the column to retrieve.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn column(&self, idx: usize) -> &Vec<Element<'a>> {
        &self.columns[idx]
    }

    pub fn record(&self, idx: usize) -> Record<'a> {
        Record::new(self.columns.iter().map(|col| col[idx].clone()).collect())
    }
}
