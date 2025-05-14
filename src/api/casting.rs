use polars::{
    frame::DataFrame,
    prelude::{Column, NamedFrom},
    series::Series,
};
use pyo3::{exceptions::PyValueError, PyResult};
use pyo3_polars::{error::PyPolarsErr, PyDataFrame};

use crate::{
    distances::{CachedDistanceCalculator, InternalDistanceMetricConfig},
    engine::{EngineConfig, TrackingEngine},
    frame::{Element, Frame},
    normalization::InternalNormalizationConfig,
    resolvers::{BestMatchResolvingStrategy, Resolver, ResolvingStrategy, SimpleResolvingStrategy},
    trackers::{InternalTrackerConfig, TrackerMemoryConfig, TrackerRecordScorerConfig},
    word::Word,
};

use super::{
    config::{MemoryConfig, RecordScorerConfig},
    DistanceMetricConfig, ElementType, FieldSchema, NormalizationConfig, RecordSchema,
    ResolverConfig, TrackerConfig, TrackingConfig,
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

/// Casts a frame to a polars dataframe.
///
/// # Errors
/// Returns PyPolarsErr if the frame cannot be cast to a DataFrame.
pub fn cast_to_dataframe(record_schema: &RecordSchema, frame: &Frame) -> PyResult<PyDataFrame> {
    let mut columns = Vec::new();
    for (i, field_schema) in record_schema.fields.iter().enumerate() {
        let column = frame.column(i);
        let series = match field_schema.dtype {
            ElementType::String => Column::new(
                field_schema.name.as_str().into(),
                column
                    .iter()
                    .map(|e| e.as_word().map(|w| w.raw.clone()))
                    .collect::<Vec<_>>(),
            ),
            ElementType::MultiStrings => {
                let v = column
                    .iter()
                    .map(|e| {
                        Series::new(
                            "".into(),
                            e.as_multiword()
                                .iter()
                                .map(|w| w.raw.as_str())
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>();

                Column::new(field_schema.name.as_str().into(), v)
            }
        };
        columns.push(series);
    }

    let df = DataFrame::new(columns).map_err(PyPolarsErr::from)?;

    Ok(PyDataFrame(df))
}

/// Get an optional attribute from a configuration.
///
/// # Errors
/// Returns PyValueError if the attribute is missing.
fn get_optional_attribute<T>(value: Option<T>, attribute: &str, context: &str) -> PyResult<T> {
    value.ok_or_else(|| {
        PyValueError::new_err(format!("Missing {} attribute in {}", attribute, context))
    })
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
        cast_engine_config(config)?,
        build_resolver(&config.resolver)?,
        build_distance_calculators(&config.distance_metric, record_schema)?,
    ))
}

/// Builds a resolver from the given configuration.
///
/// # Errors
/// Returns PyValueError if the configuration is invalid.
fn build_resolver(resolver_config: &ResolverConfig) -> PyResult<Resolver> {
    let resolving_strategy: Box<dyn ResolvingStrategy> =
        match resolver_config.resolving_strategy.as_str() {
            "simple" => Box::new(SimpleResolvingStrategy {}),
            "best-match" => Box::new(BestMatchResolvingStrategy {}),
            v => {
                return Err(PyValueError::new_err(format!(
                    "Invalid resolving strategy: {}",
                    v
                )));
            }
        };

    Ok(Resolver::new(resolving_strategy))
}

fn cast_distance_metric_config(
    distance_metric_config: &DistanceMetricConfig,
) -> PyResult<InternalDistanceMetricConfig> {
    match distance_metric_config.metric.as_str() {
        "lv" => Ok(InternalDistanceMetricConfig::Lv(
            distance_metric_config.use_sigmoid,
        )),
        "lv_opti" => Ok(InternalDistanceMetricConfig::LvOpti(
            distance_metric_config.use_sigmoid,
        )),
        "lv_edit" => {
            let weights = get_optional_attribute(
                distance_metric_config.lv_edit_weights.clone(),
                "lv_edit_weights",
                "DistanceMetricConfig",
            )?;
            if weights.len() != 3 {
                return Err(PyValueError::new_err(
                    "lv_edit_weights attribute must have 3 weights in DistanceMetricConfig",
                ));
            }

            Ok(InternalDistanceMetricConfig::LvEdit(
                weights[0],
                weights[1],
                weights[2],
                distance_metric_config.use_sigmoid,
            ))
        }
        "lv_substring" => Ok(InternalDistanceMetricConfig::LvSubstring(
            get_optional_attribute(
                distance_metric_config.lv_substring_weight,
                "lv_substring_weight",
                "DistanceMetricConfig",
            )?,
            distance_metric_config.use_sigmoid,
        )),
        "lv_multiword" => {
            let separator = get_optional_attribute(
                distance_metric_config.lv_multiword_separator.clone(),
                "lv_multiword_separator",
                "DistanceMetricConfig",
            )?;

            Ok(InternalDistanceMetricConfig::LvMultiWord(
                Word::string_to_grapheme(separator.as_str()),
                distance_metric_config.use_sigmoid,
            ))
        }
        v => Err(PyValueError::new_err(format!(
            "Invalid distance metric: {}",
            v
        ))),
    }
}

/// Builds a distance calculator from the given configuration.
///
/// # Errors
/// Returns PyValueError if the configuration is invalid.
pub fn build_distance_calculator(
    distance_metric_config: &DistanceMetricConfig,
) -> PyResult<CachedDistanceCalculator> {
    let internal_distance_metric_config = cast_distance_metric_config(distance_metric_config)?;
    Ok(CachedDistanceCalculator::new(
        internal_distance_metric_config.make_metric(),
        distance_metric_config.caching_threshold,
    ))
}

/// Builds a list of distance calculators from the given configuration and record schema.
///
/// # Errors
/// Returns PyValueError if the configuration is invalid.
fn build_distance_calculators(
    distance_metric_config: &DistanceMetricConfig,
    record_schema: &RecordSchema,
) -> PyResult<Vec<CachedDistanceCalculator>> {
    let internal_distance_metric_config = cast_distance_metric_config(distance_metric_config)?;
    let mut distance_calculators = Vec::new();
    for _ in record_schema.fields.iter() {
        let distance_calculator = CachedDistanceCalculator::new(
            internal_distance_metric_config.make_metric(),
            distance_metric_config.caching_threshold,
        );
        distance_calculators.push(distance_calculator);
    }
    Ok(distance_calculators)
}

/// Cast a RecordScorerConfig to a TrackerRecordScorer.
///
/// # Errors
/// Returns PyValueError if the configuration is invalid.
fn cast_record_scorer_config(
    record_scorer_config: &RecordScorerConfig,
) -> PyResult<TrackerRecordScorerConfig> {
    Ok(match record_scorer_config.record_scorer.as_str() {
        "average" => TrackerRecordScorerConfig::Average,
        "weighted-average" => TrackerRecordScorerConfig::WeightedAverage(
            get_optional_attribute(
                record_scorer_config.weights.clone(),
                "weights",
                "RecordScorerConfig",
            )?,
            get_optional_attribute(
                record_scorer_config.min_weight_ratio,
                "min_weight_ratio",
                "RecordScorerConfig",
            )?,
        ),
        "weighted-quadratic" => TrackerRecordScorerConfig::WeightedQuadratic(
            get_optional_attribute(
                record_scorer_config.weights.clone(),
                "weights",
                "RecordScorerConfig",
            )?,
            get_optional_attribute(
                record_scorer_config.min_weight_ratio,
                "min_weight_ratio",
                "RecordScorerConfig",
            )?,
        ),
        v => {
            return Err(PyValueError::new_err(format!(
                "Invalid record scorer: {}",
                v
            )))
        }
    })
}

/// Cast a TrackingConfig to an EngineConfig.
///
/// # Errors
/// Returns PyValueError if the configuration is invalid.
fn cast_engine_config(config: &TrackingConfig) -> PyResult<EngineConfig> {
    Ok(EngineConfig {
        num_threads: config.num_threads,
        tracker_config: cast_tracker_config(&config.tracker)?,
    })
}

fn cast_multiword_memory_config(
    memory_config: &MemoryConfig,
    tracker_memory_config: TrackerMemoryConfig,
) -> PyResult<TrackerMemoryConfig> {
    Ok(TrackerMemoryConfig::MultiWord(
        Box::new(tracker_memory_config),
        cast_distance_metric_config(&get_optional_attribute(
            memory_config.multiword_distance_metric.clone(),
            "multiword_distance_metric",
            "MemoryConfig",
        )?)?,
        get_optional_attribute(
            memory_config.multiword_threshold_match,
            "multiword_threshold_match",
            "MemoryConfig",
        )?,
    ))
}

fn cast_memory_config(memory_config: &MemoryConfig) -> PyResult<TrackerMemoryConfig> {
    Ok(match memory_config.memory_strategy.as_str() {
        "bruteforce" => TrackerMemoryConfig::BruteForce,
        "mostfrequent" => TrackerMemoryConfig::MostFrequent,
        "median" => TrackerMemoryConfig::Median,
        "ls-bruteforce" => {
            TrackerMemoryConfig::LongShortTerm(Box::new(TrackerMemoryConfig::BruteForce))
        }
        "ls-mostfrequent" => {
            TrackerMemoryConfig::LongShortTerm(Box::new(TrackerMemoryConfig::MostFrequent))
        }
        "ls-median" => TrackerMemoryConfig::LongShortTerm(Box::new(TrackerMemoryConfig::Median)),
        "mw-mostfrequent" => {
            cast_multiword_memory_config(memory_config, TrackerMemoryConfig::MostFrequent)?
        }
        "mw-median" => cast_multiword_memory_config(memory_config, TrackerMemoryConfig::Median)?,
        v => {
            return Err(PyValueError::new_err(format!(
                "Invalid tracker memory strategy: {}",
                v
            )))
        }
    })
}

/// Cast a TrackerConfig to an InternalTrackerConfig.
///
/// # Errors
/// Returns PyValueError if the configuration is invalid.
fn cast_tracker_config(tracker_config: &TrackerConfig) -> PyResult<InternalTrackerConfig> {
    let mut memory_configs = Vec::new();
    for memory_config in tracker_config.memories.iter() {
        memory_configs.push(cast_memory_config(memory_config)?);
    }

    Ok(InternalTrackerConfig {
        interest_threshold: tracker_config.interest_threshold,
        limit_no_match_streak: tracker_config.limit_no_match_streak,
        memory_configs,
        record_scorer: cast_record_scorer_config(&tracker_config.record_scorer)?,
    })
}

pub fn cast_normalization_config(
    normalization_config: &NormalizationConfig,
) -> InternalNormalizationConfig {
    InternalNormalizationConfig {
        threshold_cluster_match: normalization_config.threshold_cluster_match,
        min_cluster_size: normalization_config.min_cluster_size,
        infer_missing_clusters: normalization_config.infer_missing_clusters,
    }
}
