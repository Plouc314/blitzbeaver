use pyo3::{pyclass, pymethods};

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct TrackingConfig {
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
        tracker: TrackerConfig,
        distance_metric: DistanceMetricConfig,
        resolver: ResolverConfig,
    ) -> Self {
        Self {
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
}

#[pymethods]
impl DistanceMetricConfig {
    #[new]
    pub fn py_new(metric: String, caching_threshold: u32) -> Self {
        Self {
            metric,
            caching_threshold,
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
}

#[pymethods]
impl RecordScorerConfig {
    #[new]
    #[pyo3(signature = (record_scorer, weights=None))]
    pub fn py_new(record_scorer: String, weights: Option<Vec<f32>>) -> Self {
        Self {
            record_scorer,
            weights,
        }
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct TrackerConfig {
    #[pyo3(get)]
    pub interest_threshold: f32,
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
        memory_strategy: String,
        record_scorer: RecordScorerConfig,
    ) -> Self {
        Self {
            interest_threshold,
            memory_strategy,
            record_scorer,
        }
    }
}
