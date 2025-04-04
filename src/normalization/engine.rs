use crate::{
    api::{ElementType, RecordSchema, TrackingGraph},
    frame::{Element, Frame},
    trackers::TrackingChain,
};

use super::Normalizer;

/// NormalizationEngine
///
/// Responsible for generating new normalized frames based on the original frames
/// and the tracking graph.
///
/// It normalizes the words based on the tracking chain they appear in.
pub struct NormalizationEngine {
    frames: Vec<Frame>,
    normalized_frames: Vec<Frame>,
    record_schema: RecordSchema,
    tracking_graph: TrackingGraph,
    normalizer: Normalizer,
}

impl NormalizationEngine {
    pub fn new(
        frames: Vec<Frame>,
        tracking_graph: TrackingGraph,
        record_schema: RecordSchema,
        normalizer: Normalizer,
    ) -> Self {
        Self {
            normalized_frames: frames.clone(),
            frames,
            record_schema,
            tracking_graph,
            normalizer,
        }
    }

    fn normalize_word_feature(&mut self, tracking_chain: &TrackingChain, feature_idx: usize) {
        let mut words = Vec::new();
        for node in tracking_chain.nodes.iter() {
            let frame = &self.frames[node.frame_idx];
            let element = &frame.column(feature_idx)[node.record_idx];
            words.push(element.as_word());
        }

        let normalized_words = self.normalizer.normalize_words(words);

        for (i, node) in tracking_chain.nodes.iter().enumerate() {
            let frame = &mut self.normalized_frames[node.frame_idx];
            frame.mut_column(feature_idx)[node.record_idx] = match normalized_words[i] {
                None => Element::None,
                Some(ref word) => Element::Word(word.clone()),
            };
        }
    }

    fn normalize_multiword_feature(&mut self, tracking_chain: &TrackingChain, feature_idx: usize) {
        let mut words = Vec::new();
        for node in tracking_chain.nodes.iter() {
            let frame = &self.frames[node.frame_idx];
            let element = &frame.column(feature_idx)[node.record_idx];
            words.push(element.as_multiword());
        }

        let normalized_words = self.normalizer.normalize_multi_words(words);

        for (i, node) in tracking_chain.nodes.iter().enumerate() {
            let frame = &mut self.normalized_frames[node.frame_idx];
            frame.mut_column(feature_idx)[node.record_idx] =
                Element::MultiWords(normalized_words[i].clone());
        }
    }

    fn normalize_tracking_chain(&mut self, tracking_chain: &TrackingChain) {
        let record_schema = self.record_schema.clone();
        for (feature_idx, field) in record_schema.fields.iter().enumerate() {
            match field.dtype {
                ElementType::String => {
                    self.normalize_word_feature(tracking_chain, feature_idx);
                }
                ElementType::MultiStrings => {
                    self.normalize_multiword_feature(tracking_chain, feature_idx);
                }
            }
        }
    }

    pub fn normalize(mut self) -> Vec<Frame> {
        let tracker_ids = self
            .tracking_graph
            .root
            .outs
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<u64>>();
        for id in tracker_ids {
            let tracking_chain = self.tracking_graph.build_tracking_chain(id);
            self.normalize_tracking_chain(&tracking_chain);
        }

        self.normalized_frames
    }
}
