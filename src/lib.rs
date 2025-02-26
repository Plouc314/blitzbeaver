use pyo3::prelude::*;

mod api;
mod benchmark;
mod distances;
mod evaluation;
mod frame;
mod id;
mod resolvers;
mod trackers;
mod tracking_engine;
mod word;

#[pymodule]
fn blitzbeaver(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<api::RecordSchema>()?;
    m.add_class::<api::FieldSchema>()?;
    m.add_class::<api::ElementType>()?;

    m.add_class::<api::TrackingConfig>()?;
    m.add_class::<api::ResolverConfig>()?;
    m.add_class::<api::DistanceMetricConfig>()?;
    m.add_class::<api::TrackerConfig>()?;
    m.add_class::<api::SimpleTrackerConfig>()?;

    m.add_class::<api::ChainNode>()?;
    m.add_class::<api::GraphNode>()?;
    m.add_class::<api::TrackingGraph>()?;

    m.add_function(wrap_pyfunction!(api::test_tracking_engine, m)?)?;

    m.add_function(wrap_pyfunction!(api::evaluate_tracking_chain_length, m)?)?;
    m.add_function(wrap_pyfunction!(
        api::evaluate_tracking_graph_properties,
        m
    )?)?;
    Ok(())
}
