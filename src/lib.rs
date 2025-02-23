use pyo3::prelude::*;

mod api;
mod benchmark;
mod distances;
mod frame;
mod id;
mod resolvers;
mod trackers;
mod tracking_engine;
mod word;

/// A Python module implemented in Rust.
#[pymodule]
fn blitzbeaver(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<api::RecordSchema>()?;
    m.add_class::<api::FieldSchema>()?;
    m.add_class::<api::ElementType>()?;

    m.add_function(wrap_pyfunction!(api::test_tracking_engine, m)?)?;

    #[cfg(feature = "benchmark")]
    {
        // m.add_function(wrap_pyfunction!(
        //     benchmark::distance::benchmark_distance_functions,
        //     m
        // )?)?;

        // m.add_function(wrap_pyfunction!(
        //     benchmark::distance::benchmark_feature_distance_calculator,
        //     m
        // )?)?;

        // m.add_function(wrap_pyfunction!(
        //     benchmark::distance::benchmark_feature_distance_calculator_second_pass,
        //     m
        // )?)?;

        // m.add_function(wrap_pyfunction!(
        //     benchmark::distance::benchmark_feature_distance_calculator_multi_pass,
        //     m
        // )?)?;
    }
    Ok(())
}
