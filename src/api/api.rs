use crate::{
    distances, logger,
    normalization::{self, InternalNormalizationConfig, Normalizer},
    word::Word,
};
use pyo3::{pyfunction, PyResult};
use pyo3_polars::PyDataFrame;

use super::{
    casting, schema::RecordSchema, Diagnostics, DistanceMetricConfig, TrackingConfig, TrackingGraph,
};

#[pyfunction]
pub fn setup_logger(log_level: String) {
    logger::initialize_logger(&log_level);
}

#[pyfunction]
pub fn execute_tracking_process(
    tracking_config: &TrackingConfig,
    record_schema: &RecordSchema,
    dataframes: Vec<PyDataFrame>,
) -> PyResult<(TrackingGraph, Diagnostics)> {
    let mut frames = Vec::new();
    for i in 0..dataframes.len() {
        let frame = casting::cast_to_frame(i, record_schema, &dataframes[i])?;
        frames.push(frame);
    }

    let mut tracking_engine =
        casting::build_tracking_engine(tracking_config, record_schema, frames)?;

    for frame_idx in 1..dataframes.len() {
        log::debug!("processing frame {}...", frame_idx);
        tracking_engine.process_next_frame();
    }

    let tracking_chains = tracking_engine.stop();
    let tracking_graph =
        TrackingGraph::from_tracking_chains(tracking_engine.frames(), tracking_chains);

    Ok((tracking_graph, tracking_engine.take_diagnostics()))
}

#[pyfunction]
pub fn compute_median_word(words: Vec<String>) -> Option<String> {
    let words = words
        .into_iter()
        .map(|w| Word::new(w))
        .collect::<Vec<Word>>();
    let median_word = distances::compute_median_word(&words.iter().map(|w| w).collect());
    median_word.map(|w| w.raw)
}

#[pyfunction]
pub fn compute_words_clusters(
    words: Vec<String>,
    distance_metric_config: DistanceMetricConfig,
    threshold_match: f32,
) -> PyResult<Vec<Vec<String>>> {
    let words = words
        .into_iter()
        .map(|w| Word::new(w))
        .collect::<Vec<Word>>();

    let mut distance_calculator = casting::build_distance_calculator(&distance_metric_config)?;
    let clusters_sets = normalization::compute_words_clusters(
        &mut distance_calculator,
        words.iter().collect(),
        threshold_match,
    );

    let mut clusters = Vec::new();
    for cluster in clusters_sets.into_iter() {
        let mut cluster_words = Vec::new();
        for i in cluster.iter() {
            cluster_words.push(&words[i]);
        }
        clusters.push(cluster_words);
    }

    Ok(clusters
        .iter()
        .map(|cluster| cluster.iter().map(|w| w.raw.clone()).collect())
        .collect())
}

#[pyfunction]
pub fn normalize_words(
    words: Vec<Option<String>>,
    distance_metric_config: DistanceMetricConfig,
    threshold_match: f32,
    min_cluster_size: usize,
) -> PyResult<Vec<String>> {
    let words = words
        .into_iter()
        .map(|w| w.map(|w| Word::new(w)))
        .collect::<Vec<Option<Word>>>();

    let distance_calculator = casting::build_distance_calculator(&distance_metric_config)?;
    let mut normalizer = Normalizer::new(
        InternalNormalizationConfig {
            threshold_cluster_match: threshold_match,
            min_cluster_size: min_cluster_size,
        },
        distance_calculator,
    );

    let normalized_words = normalizer.normalize_words(words.iter().map(|w| w.as_ref()).collect());

    Ok(normalized_words.iter().map(|w| w.raw.clone()).collect())
}
