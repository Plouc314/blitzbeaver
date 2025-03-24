use std::{collections::HashMap, hash::Hash};

use crate::word::Word;

use super::{LvEdit, LvEditDistanceMetric};

/// Computes the median word of a list of words.
pub fn compute_median_word(words: &Vec<&Word>) -> Option<Word> {
    if words.is_empty() {
        return None;
    }

    let mut distance_metric = LvEditDistanceMetric::new(1., 1., 1., false);

    // compute most frequent length
    let mfl = most_frequent(words.iter().map(|w| w.graphemes.len())).unwrap();

    // collect words with most frequent length
    let mfl_words: Vec<&Word> = words
        .iter()
        .filter_map(|w| {
            if w.graphemes.len() == mfl {
                Some(*w)
            } else {
                None
            }
        })
        .collect();

    // align words with different length
    let mut aligned_words: Vec<Word> = Vec::new();
    for word in words.iter() {
        if word.graphemes.len() != mfl {
            let aligned_word = align_word(&mut distance_metric, word, &mfl_words);
            aligned_words.push(aligned_word);
        }
    }

    // collect all words
    let mut words = Vec::new();
    words.extend(mfl_words);
    words.extend(aligned_words.iter().map(|w| w));

    // compute median with most frequent graphemes
    Some(compute_most_frequent_graphemes_word(words))
}

/// Returns the most frequent element in an iterator or None if the iterator is empty.
///
/// In there are multiple elements with the same frequency, no guarantee is made about
/// which one is returned.
fn most_frequent<T: Hash + Eq + Clone>(iter: impl Iterator<Item = T>) -> Option<T> {
    let mut max_count = 0;
    let mut mfv = None;
    let mut counts = HashMap::new();
    for v in iter {
        let mut count = 1;
        counts
            .entry(v.clone())
            .and_modify(|c| {
                *c += 1;
                count = *c;
            })
            .or_insert(1);

        if count > max_count {
            max_count = count;
            mfv = Some(v);
        }
    }
    mfv
}

/// Performs the add and delete operations of the edits to the word.
///
/// For the add operation, put a placeholder grapheme (0) instead
/// of the one that should be added.
///
/// This will result in a new word with the same length as the target word
/// but not necessarily the same graphemes.
fn perform_add_del_edits(w: &Word, edits: &Vec<LvEdit>) -> Word {
    let mut graphemes = w.graphemes.clone();
    let mut idx_shift: i64 = 0;
    for edit in edits {
        match edit {
            LvEdit::Add(idx, _) => {
                // add a placeholder grapheme
                graphemes.insert(*idx, 0);
                idx_shift += 1;
            }
            LvEdit::Del(idx) => {
                let idx = (*idx as i64 + idx_shift) as usize;
                graphemes.remove(idx);
                idx_shift -= 1;
            }
            LvEdit::Sub(_, _) => {}
        }
    }
    Word::from_graphemes(graphemes)
}

/// Aligns a word with a random word from the most frequent length words.
///
/// Alignment is done by adding/deleting graphemes according to the Levenshtein distance.
fn align_word(
    distance_metric: &mut LvEditDistanceMetric,
    word: &Word,
    mfl_words: &Vec<&Word>,
) -> Word {
    let idx = rand::random_range(..mfl_words.len());
    let target = mfl_words[idx];
    let edits = distance_metric.compute_edits(word, target);
    perform_add_del_edits(word, &edits)
}

/// Computes a new word for which every grapheme is the most frequent grapheme
/// at that position in the words.
///
/// All the words must have the same length.
fn compute_most_frequent_graphemes_word(words: Vec<&Word>) -> Word {
    let length = words[0].graphemes.len();
    let mut graphemes = Vec::with_capacity(length);
    for i in 0..length {
        let grapheme = most_frequent(words.iter().filter_map(|w| {
            // do not take into account the placeholder grapheme
            if w.graphemes[i] == 0 {
                None
            } else {
                Some(w.graphemes[i])
            }
        }))
        .unwrap();
        graphemes.push(grapheme);
    }
    Word::from_graphemes(graphemes)
}
