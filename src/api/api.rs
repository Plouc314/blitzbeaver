use crate::{distances, logger, word::Word};
use pyo3::{pyfunction, PyResult};
use pyo3_polars::PyDataFrame;

use super::{casting, schema::RecordSchema, Diagnostics, TrackingConfig, TrackingGraph};

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
