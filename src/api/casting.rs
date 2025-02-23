use polars::series::Series;
use pyo3::{exceptions::PyValueError, PyResult};
use pyo3_polars::{error::PyPolarsErr, PyDataFrame};

use crate::{
    distances::{
        CachedDistanceCalculator, CachedDistanceCalculatorWord, DistanceMetric, LvDistanceMetric,
        LvOptiDistanceMetric,
    },
    frame::{Element, Frame},
    resolvers::{Resolver, SimpleResolvingStrategy},
    trackers::{SimpleTracker, Tracker},
    tracking_engine::TrackingEngine,
    word::Word,
};

use super::{
    DistanceMetricConfig, ElementType, FieldSchema, RecordSchema, ResolverConfig, TrackerConfig,
    TrackingConfig,
};

fn cast_to_frame_column<'a>(
    field_schema: &FieldSchema,
    serie: &'a Series,
) -> PyResult<Vec<Element<'a>>> {
    match &field_schema.dtype {
        ElementType::String => Ok(serie
            .str()
            .map_err(PyPolarsErr::from)?
            .iter()
            .map(|v| match v {
                None => Element::None,
                Some(v) => Element::Word(Word::new(v)),
            })
            .collect()),
        ElementType::MultiStrings => Ok(serie
            .list()
            .map_err(PyPolarsErr::from)?
            .iter()
            .map(|v| match v {
                Some(v) => {
                    unimplemented!()
                }
                None => Element::None,
            })
            .collect()),
    }
}

pub fn cast_to_frame<'a>(
    frame_idx: usize,
    record_schema: &RecordSchema,
    dataframe: &'a PyDataFrame,
) -> PyResult<Frame<'a>> {
    let mut columns = Vec::new();
    for field_schema in record_schema.fields.iter() {
        let column = dataframe
            .0
            .column(&field_schema.name)
            .map_err(PyPolarsErr::from)?;
        let series = column.as_series().ok_or(PyValueError::new_err(
            "Internal error: invalid polars column",
        ))?;

        columns.push(cast_to_frame_column(field_schema, series)?);
    }

    Ok(Frame::new(frame_idx, columns))
}

pub fn build_tracking_engine<'a>(
    config: &TrackingConfig,
    record_schema: &RecordSchema,
    frames: Vec<Frame<'a>>,
) -> PyResult<TrackingEngine<'a, impl Fn() -> Box<dyn Tracker<'a>>>> {
    Ok(TrackingEngine::new(
        frames,
        build_resolver(&config.resolver)?,
        build_distance_calculators(&config.distance_metric, record_schema)?,
        build_tracker_builder(&config.tracker)?,
    ))
}

fn build_resolver<'a>(resolver_config: &ResolverConfig) -> PyResult<Resolver<'a>> {
    let resolving_strategy = match resolver_config.resolving_strategy.as_str() {
        "simple" => Box::new(SimpleResolvingStrategy {}),
        v => {
            return Err(PyValueError::new_err(format!(
                "Invalid resolving strategy: {}",
                v
            )));
        }
    };

    Ok(Resolver::new(resolving_strategy))
}

fn build_distance_metric<'a>(
    distance_metric_config: &DistanceMetricConfig,
) -> PyResult<Box<dyn DistanceMetric<Word<'a>>>> {
    match distance_metric_config.metric.as_str() {
        "lv" => Ok(Box::new(LvDistanceMetric::new())),
        "lvopti" => Ok(Box::new(LvOptiDistanceMetric::new())),
        v => Err(PyValueError::new_err(format!(
            "Invalid distance metric: {}",
            v
        ))),
    }
}

fn build_distance_calculator<'a>(
    distance_metric_config: &DistanceMetricConfig,
    field_schema: &FieldSchema,
) -> PyResult<CachedDistanceCalculator<'a>> {
    Ok(match field_schema.dtype {
        ElementType::String => CachedDistanceCalculator::Word(CachedDistanceCalculatorWord::new(
            build_distance_metric(distance_metric_config)?,
            distance_metric_config.caching_threshold,
        )),
        ElementType::MultiStrings => {
            unimplemented!()
        }
    })
}

fn build_distance_calculators<'a>(
    distance_metric_config: &DistanceMetricConfig,
    record_schema: &RecordSchema,
) -> PyResult<Vec<CachedDistanceCalculator<'a>>> {
    let mut distance_calculators = Vec::new();
    for field in record_schema.fields.iter() {
        let distance_calculator = build_distance_calculator(distance_metric_config, field)?;
        distance_calculators.push(distance_calculator);
    }
    Ok(distance_calculators)
}

fn build_tracker_builder<'a>(
    tracker_config: &TrackerConfig,
) -> PyResult<impl Fn() -> Box<dyn Tracker<'a>>> {
    match tracker_config.tracker_type.as_str() {
        "simple" => {
            let config = match &tracker_config.simple_tracker {
                Some(config) => config.clone(),
                None => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid simple tracker config"
                    )));
                }
            };
            Ok(move || {
                let tracker = Box::new(SimpleTracker::new(config.clone()));
                // Safety: there is some unclear lifetime issue with the tracker
                // that is not resolved yet. This is a temporary workaround.
                unsafe {
                    std::mem::transmute::<Box<dyn Tracker<'static>>, Box<dyn Tracker<'a>>>(tracker)
                }
            })
        }
        v => Err(PyValueError::new_err(format!(
            "Invalid tracker type: {}",
            v
        ))),
    }
}
