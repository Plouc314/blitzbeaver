use log::info;
use pyo3::{pyfunction, PyResult};
use pyo3_polars::PyDataFrame;
use crate::logger;

use super::{casting, schema::RecordSchema, TrackingConfig, TrackingGraph};

#[pyfunction]
pub fn test_tracking_engine(
    tracking_config: &TrackingConfig,
    record_schema: &RecordSchema,
    dataframes: Vec<PyDataFrame>,
    log_level: String,
) -> PyResult<TrackingGraph> {
    logger::initialize_logger(&log_level);
    wrapper(tracking_config, record_schema, &dataframes)
}

fn wrapper<'a>(
    tracking_config: &TrackingConfig,
    record_schema: &RecordSchema,
    dataframes: &'a Vec<PyDataFrame>,
) -> PyResult<TrackingGraph> {
    let mut frames = Vec::new();
    for i in 0..dataframes.len() {
        let frame = casting::cast_to_frame(i, record_schema, &dataframes[i])?;
        frames.push(frame);
    }

    let mut tracking_engine =
        casting::build_tracking_engine(tracking_config, record_schema, frames)?;

    tracking_engine.initialize_trackers();

    for frame_idx in 1..dataframes.len() {
        tracking_engine.process_next_frame();
    }

    let tracking_chains = tracking_engine.collect_tracking_chains();

    Ok(TrackingGraph::from_tracking_chains(
        tracking_engine.frames(),
        tracking_chains,
    ))
}
