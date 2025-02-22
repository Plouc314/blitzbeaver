use polars::series::Series;
use pyo3::{pyfunction, PyResult};
use pyo3_polars::{error::PyPolarsErr, PyDataFrame};

use crate::{
    distances::{CachedDistanceCalculator, CachedDistanceCalculatorWord, LvOptiDistance},
    frame::{Element, Frame},
    resolvers::{Resolver, SimpleResolvingStrategy},
    trackers::{SimpleTracker, SimpleTrackerConfig, Tracker},
    tracking_engine::TrackingEngine,
    word::Word,
};

use super::schema::{FieldSchema, RecordSchema};

fn pyserie_to_vec<'a>(field_schema: &FieldSchema, serie: &'a Series) -> PyResult<Vec<Element<'a>>> {
    Ok(serie
        .str()
        .map_err(PyPolarsErr::from)?
        .iter()
        .map(|v| match v {
            None => Element::None,
            Some(v) => Element::Word(Word::new(v)),
        })
        .collect())
}

fn pydataframe_to_frame<'a>(
    frame_idx: usize,
    record_schema: &RecordSchema,
    dataframe: &'a PyDataFrame,
    column_names: &Vec<String>,
) -> PyResult<Frame<'a>> {
    let mut columns = Vec::new();
    for (column_name, field_schema) in column_names.iter().zip(record_schema.fields.iter()) {
        let column = dataframe
            .0
            .column(&column_name)
            .map_err(PyPolarsErr::from)?;
        columns.push(pyserie_to_vec(
            field_schema,
            column.as_series().expect("invalid polars column type"),
        )?);
    }

    Ok(Frame::new(frame_idx, columns))
}

#[pyfunction]
pub fn test_tracking_engine(
    record_schema: &RecordSchema,
    dataframes: Vec<PyDataFrame>,
    column_names: Vec<String>,
) -> PyResult<()> {
    wrapper(record_schema, &dataframes, column_names)
}

fn wrapper<'a>(
    record_schema: &RecordSchema,
    dataframes: &'a Vec<PyDataFrame>,
    column_names: Vec<String>,
) -> PyResult<()> {
    let mut frames = Vec::new();
    for i in 0..dataframes.len() {
        let frame = pydataframe_to_frame(i, record_schema, &dataframes[i], &column_names)?;
        frames.push(frame);
    }

    let resolving_strategy = SimpleResolvingStrategy {};

    let resolver = Resolver::new(Box::new(resolving_strategy));

    let distance_calculators: Vec<CachedDistanceCalculator> = (0..column_names.len())
        .map(|_| {
            CachedDistanceCalculator::Word(CachedDistanceCalculatorWord::new(
                Box::new(LvOptiDistance::new()),
                4,
            ))
        })
        .collect();

    let tracker_builder = || {
        let config = SimpleTrackerConfig {
            interest_threshold: 0.7,
        };
        let tracker = Box::new(SimpleTracker::new(config));
        // Safety: there is some unclear lifetime issue with the tracker
        // that is not resolved yet. This is a temporary workaround.
        unsafe { std::mem::transmute::<Box<dyn Tracker<'static>>, Box<dyn Tracker<'a>>>(tracker) }
    };

    let mut tracking_engine =
        TrackingEngine::new(frames, resolver, distance_calculators, tracker_builder);

    tracking_engine.initialize_trackers();

    for frame_idx in 1..dataframes.len() {
        tracking_engine.process_next_frame();
    }

    let tracking_chains = tracking_engine.collect_tracking_chains();

    Ok(())
}
