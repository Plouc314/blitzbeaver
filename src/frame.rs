use crate::word::Word;

pub struct Frame<'a> {
    columns: Vec<Vec<Option<Word<'a>>>>,
}

impl<'a> Frame<'a> {
    /// Returns the number of records
    pub fn nrecs(&self) -> usize {
        self.columns[0].len()
    }

    /// Returns the number of features
    pub fn nfeats(&self) -> usize {
        self.columns.len()
    }

    pub fn columns(&self) -> &Vec<Vec<Option<Word<'a>>>> {
        &self.columns
    }
}
