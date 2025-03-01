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
    trackers::{InternalTrackerConfig, TrackerMemoryStrategy},
    tracking_engine::TrackingEngine,
    word::Word,
};

use super::{
    DistanceMetricConfig, ElementType, FieldSchema, RecordSchema, ResolverConfig, TrackerConfig,
    TrackingConfig,
};

/// Casts a polars series to a vector of Word elements.
///
/// # Errors
/// Returns a PyPolarsErr if the series cannot be cast to a string series.
fn cast_to_string_column(serie: &Series) -> PyResult<Vec<Element>> {
    Ok(serie
        .str()
        .map_err(PyPolarsErr::from)?
        .iter()
        .map(|v| match v {
            None => Element::None,
            Some(v) => Element::Word(Word::new(v.to_string())),
        })
        .collect())
}

/// Casts a polars series to a vector of MultiWords elements.
///
/// # Errors
/// Returns a PyPolarsErr if the series cannot be cast to a list of string series.
/// Returns a PyValueError if a None value is found in the list.
fn cast_to_multistrings_column(serie: &Series) -> PyResult<Vec<Element>> {
    let mut elements = Vec::new();

    for cell in serie.list().map_err(PyPolarsErr::from)?.into_iter() {
        match cell {
            Some(cell) => {
                let mut words = Vec::new();

                for v in cell.str().map_err(PyPolarsErr::from)?.into_iter() {
                    match v {
                        Some(value) => {
                            words.push(Word::new(value.to_string()));
                        }
                        None => {
                            return Err(PyValueError::new_err(format!(
                                "None value in list[str] cell: {:?}",
                                cell
                            )));
                        }
                    }
                }
                elements.push(Element::MultiWords(words));
            }
            None => {
                elements.push(Element::None);
            }
        }
    }
    Ok(elements)
}

/// Casts a polars series to a vector of elements based on the field schema.
///
/// # Errors
/// Returns PyPolarsErr or PyValueError if the series cannot be cast to the specified type.
fn cast_to_frame_column(field_schema: &FieldSchema, serie: &Series) -> PyResult<Vec<Element>> {
    match &field_schema.dtype {
        ElementType::String => cast_to_string_column(serie),
        ElementType::MultiStrings => cast_to_multistrings_column(serie),
    }
}

/// Casts a polars dataframe to a Frame.
///
/// # Errors
/// Returns PyPolarsErr or PyValueError if the dataframe cannot be cast to a Frame.
pub fn cast_to_frame(
    frame_idx: usize,
    record_schema: &RecordSchema,
    dataframe: &PyDataFrame,
) -> PyResult<Frame> {
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

/// Builds a tracking engine from the given configuration and frames.
///
/// # Errors
/// Returns PyValueError if the configuration is invalid.
pub fn build_tracking_engine(
    config: &TrackingConfig,
    record_schema: &RecordSchema,
    frames: Vec<Frame>,
) -> PyResult<TrackingEngine> {
    Ok(TrackingEngine::new(
        frames,
        cast_tracker_config(&config.tracker)?,
        build_resolver(&config.resolver)?,
        build_distance_calculators(&config.distance_metric, record_schema)?,
    ))
}

/// Builds a resolver from the given configuration.
///
/// # Errors
/// Returns PyValueError if the configuration is invalid.
fn build_resolver(resolver_config: &ResolverConfig) -> PyResult<Resolver> {
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

/// Builds a distance metric from the given configuration.
///
/// # Errors
/// Returns PyValueError if the configuration is invalid.
fn build_distance_metric(
    distance_metric_config: &DistanceMetricConfig,
) -> PyResult<Box<dyn DistanceMetric<Word>>> {
    match distance_metric_config.metric.as_str() {
        "lv" => Ok(Box::new(LvDistanceMetric::new())),
        "lvopti" => Ok(Box::new(LvOptiDistanceMetric::new())),
        v => Err(PyValueError::new_err(format!(
            "Invalid distance metric: {}",
            v
        ))),
    }
}

/// Builds a distance calculator from the given configuration and field schema.
///
/// # Errors
/// Returns PyValueError if the configuration is invalid.
fn build_distance_calculator(
    distance_metric_config: &DistanceMetricConfig,
    field_schema: &FieldSchema,
) -> PyResult<CachedDistanceCalculator> {
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

/// Builds a list of distance calculators from the given configuration and record schema.
///
/// # Errors
/// Returns PyValueError if the configuration is invalid.
fn build_distance_calculators(
    distance_metric_config: &DistanceMetricConfig,
    record_schema: &RecordSchema,
) -> PyResult<Vec<CachedDistanceCalculator>> {
    let mut distance_calculators = Vec::new();
    for field in record_schema.fields.iter() {
        let distance_calculator = build_distance_calculator(distance_metric_config, field)?;
        distance_calculators.push(distance_calculator);
    }
    Ok(distance_calculators)
}

/// Cast a TrackerConfig to an InternalTrackerConfig.
///
/// # Errors
/// Returns PyValueError if the configuration is invalid.
fn cast_tracker_config(tracker_config: &TrackerConfig) -> PyResult<InternalTrackerConfig> {
    Ok(InternalTrackerConfig {
        interest_threshold: tracker_config.interest_threshold,
        memory_strategy: match tracker_config.memory_strategy.as_str() {
            "bruteforce" => TrackerMemoryStrategy::BruteForce,
            "mostfrequent" => TrackerMemoryStrategy::MostFrequent,
            "median" => TrackerMemoryStrategy::Median,
            "lsbruteforce" => TrackerMemoryStrategy::LSBruteForce,
            "lsmostfrequent" => TrackerMemoryStrategy::LSMostFrequent,
            "lsmedian" => TrackerMemoryStrategy::LSMedian,
            v => {
                return Err(PyValueError::new_err(format!(
                    "Invalid tracker memory strategy: {}",
                    v
                )))
            }
        },
    })
}
