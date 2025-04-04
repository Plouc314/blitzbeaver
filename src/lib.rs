use pyo3::prelude::*;

mod api;
mod benchmark;
mod distances;
mod engine;
mod evaluation;
mod frame;
mod histogram;
mod id;
mod logger;
mod normalization;
mod resolvers;
mod trackers;
mod word;

#[pymodule]
fn blitzbeaver(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // frame
    m.add_class::<api::RecordSchema>()?;
    m.add_class::<api::FieldSchema>()?;
    m.add_class::<api::ElementType>()?;

    // config
    m.add_class::<api::TrackingConfig>()?;
    m.add_class::<api::ResolverConfig>()?;
    m.add_class::<api::DistanceMetricConfig>()?;
    m.add_class::<api::MemoryConfig>()?;
    m.add_class::<api::RecordScorerConfig>()?;
    m.add_class::<api::TrackerConfig>()?;
    m.add_class::<api::NormalizationConfig>()?;

    // tracking graph
    m.add_class::<api::ChainNode>()?;
    m.add_class::<api::GraphNode>()?;
    m.add_class::<api::TrackingGraph>()?;

    // diagnostics
    m.add_class::<api::TrackerRecordDiagnostics>()?;
    m.add_class::<api::TrackerFrameDiagnostics>()?;
    m.add_class::<api::TrackerDiagnostics>()?;
    m.add_class::<api::ResolvingDiagnostics>()?;
    m.add_class::<api::Diagnostics>()?;

    m.add_class::<api::BeaverFile>()?;

    m.add_function(wrap_pyfunction!(api::setup_logger, m)?)?;
    m.add_function(wrap_pyfunction!(api::execute_tracking_process, m)?)?;
    m.add_function(wrap_pyfunction!(api::execute_normalization_process, m)?)?;
    m.add_function(wrap_pyfunction!(api::compute_median_word, m)?)?;
    m.add_function(wrap_pyfunction!(api::compute_words_clusters, m)?)?;
    m.add_function(wrap_pyfunction!(api::normalize_words, m)?)?;

    // evaluation
    m.add_function(wrap_pyfunction!(api::evaluate_tracking_chain_length, m)?)?;
    m.add_function(wrap_pyfunction!(
        api::evaluate_tracking_graph_properties,
        m
    )?)?;
    Ok(())
}
