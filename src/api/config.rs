use pyo3::{pyclass, pymethods};

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct TrackingConfig {
    #[pyo3(get)]
    pub num_threads: usize,
    #[pyo3(get)]
    pub tracker: TrackerConfig,
    #[pyo3(get)]
    pub distance_metric: DistanceMetricConfig,
    #[pyo3(get)]
    pub resolver: ResolverConfig,
}

#[pymethods]
impl TrackingConfig {
    #[new]
    pub fn py_new(
        num_threads: usize,
        tracker: TrackerConfig,
        distance_metric: DistanceMetricConfig,
        resolver: ResolverConfig,
    ) -> Self {
        Self {
            num_threads,
            tracker,
            distance_metric,
            resolver,
        }
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct ResolverConfig {
    #[pyo3(get)]
    pub resolving_strategy: String,
}

#[pymethods]
impl ResolverConfig {
    #[new]
    pub fn py_new(resolving_strategy: String) -> Self {
        Self { resolving_strategy }
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct DistanceMetricConfig {
    #[pyo3(get)]
    pub metric: String,
    #[pyo3(get)]
    pub caching_threshold: u32,
    #[pyo3(get)]
    pub lv_edit_weights: Option<Vec<f32>>,
    #[pyo3(get)]
    pub lv_substring_weight: Option<f32>,
    #[pyo3(get)]
    pub lv_multiword_separator: Option<String>,
}

#[pymethods]
impl DistanceMetricConfig {
    #[new]
    #[pyo3(signature = (metric, caching_threshold, lv_edit_weights=None, lv_substring_weight=None, lv_multiword_separator=None))]
    pub fn py_new(
        metric: String,
        caching_threshold: u32,
        lv_edit_weights: Option<Vec<f32>>,
        lv_substring_weight: Option<f32>,
        lv_multiword_separator: Option<String>,
    ) -> Self {
        Self {
            metric,
            caching_threshold,
            lv_edit_weights,
            lv_substring_weight,
            lv_multiword_separator,
        }
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct RecordScorerConfig {
    #[pyo3(get)]
    pub record_scorer: String,
    #[pyo3(get)]
    pub weights: Option<Vec<f32>>,
    #[pyo3(get)]
    pub min_weight_ratio: Option<f32>,
}

#[pymethods]
impl RecordScorerConfig {
    #[new]
    #[pyo3(signature = (record_scorer, weights=None, min_weight_ratio=None))]
    pub fn py_new(
        record_scorer: String,
        weights: Option<Vec<f32>>,
        min_weight_ratio: Option<f32>,
    ) -> Self {
        Self {
            record_scorer,
            weights,
            min_weight_ratio,
        }
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct TrackerConfig {
    #[pyo3(get)]
    pub interest_threshold: f32,
    #[pyo3(get)]
    pub limit_no_match_streak: usize,
    #[pyo3(get)]
    pub memory_strategy: String,
    #[pyo3(get)]
    pub record_scorer: RecordScorerConfig,
}

#[pymethods]
impl TrackerConfig {
    #[new]
    pub fn py_new(
        interest_threshold: f32,
        limit_no_match_streak: usize,
        memory_strategy: String,
        record_scorer: RecordScorerConfig,
    ) -> Self {
        Self {
            interest_threshold,
            limit_no_match_streak,
            memory_strategy,
            record_scorer,
        }
    }
}
