use crate::word::Word;

static EMPTY_MULTIWORDS: Vec<Word> = Vec::new();

/// Represents an element within a `Record` or `Frame`.
///
/// An `Element` can be:
/// - Word: A single `Word`
/// - MultiWords: A collection of multiple `Word`
/// - `None`: An empty element
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Element {
    Word(Word),
    MultiWords(Vec<Word>),
    None,
}

impl Element {
    pub fn is_none(&self) -> bool {
        *self == Self::None
    }

    pub fn as_word(&self) -> Option<&Word> {
        match self {
            Self::Word(w) => Some(w),
            Self::None => None,
            Self::MultiWords(_) => panic!("Unexpected multiword element"),
        }
    }

    pub fn as_multiword(&self) -> &Vec<Word> {
        match self {
            Self::MultiWords(words) => words,
            Self::None => &EMPTY_MULTIWORDS,
            Self::Word(_) => panic!("Unexpected word element"),
        }
    }
}

/// Represents a row in a `Frame`.
///
/// A `Record` is composed of multiple `Element`s, each corresponding to a column
/// in the `Frame`.
#[derive(Default, Clone)]
pub struct Record {
    elements: Vec<Element>,
}

impl Record {
    /// Creates a new `Record` from a vector of `Element`s.
    pub fn new(elements: Vec<Element>) -> Self {
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
    pub fn element(&self, idx: usize) -> &Element {
        &self.elements[idx]
    }
}

/// Represents a structured collection of records.
///
/// The `Frame` stores data in a column-major format, meaning each column contains
/// all values for a specific feature across all records. This improves cache
/// locality when processing data column-wise.
#[derive(Clone)]
pub struct Frame {
    idx: usize,
    columns: Vec<Vec<Element>>,
}

impl Frame {
    /// Creates a new `Frame` from a vector of columns, where each column
    /// contains elements for all records.
    pub fn new(idx: usize, columns: Vec<Vec<Element>>) -> Self {
        Self { idx, columns }
    }

    /// Returns the index of the `Frame`.
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
    pub fn column(&self, idx: usize) -> &Vec<Element> {
        &self.columns[idx]
    }

    pub fn mut_column(&mut self, idx: usize) -> &mut Vec<Element> {
        &mut self.columns[idx]
    }

    /// Returns a new `Record` from the elements at the specified index across all columns.
    pub fn record(&self, idx: usize) -> Record {
        Record::new(self.columns.iter().map(|col| col[idx].clone()).collect())
    }
}
