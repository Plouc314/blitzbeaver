use pyo3::prelude::*;
use record::{FieldSchema, RecordSchema};

mod benchmark;
mod distances;
mod frame;
mod record;
mod tracker;
mod tracking_engine;
mod word;

/// A Python module implemented in Rust.
#[pymodule]
fn blitzbeaver(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RecordSchema>()?;
    m.add_class::<FieldSchema>()?;

    #[cfg(feature = "benchmark")]
    {
        m.add_function(wrap_pyfunction!(
            benchmark::distance::benchmark_distance_functions,
            m
        )?)?;

        m.add_function(wrap_pyfunction!(
            benchmark::distance::benchmark_feature_distance_calculator,
            m
        )?)?;

        m.add_function(wrap_pyfunction!(
            benchmark::distance::benchmark_feature_distance_calculator_second_pass,
            m
        )?)?;

        m.add_function(wrap_pyfunction!(
            benchmark::distance::benchmark_feature_distance_calculator_multi_pass,
            m
        )?)?;
    }
    Ok(())
}
