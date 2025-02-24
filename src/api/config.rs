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
pub struct TrackerConfig {
    #[pyo3(get)]
    pub tracker_type: String,
    #[pyo3(get)]
    pub simple_tracker: Option<SimpleTrackerConfig>,
}

#[pymethods]
impl TrackerConfig {
    #[new]
    #[pyo3(signature = (tracker_type, simple_tracker=None))]
    pub fn py_new(tracker_type: String, simple_tracker: Option<SimpleTrackerConfig>) -> Self {
        Self {
            tracker_type,
            simple_tracker,
        }
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct SimpleTrackerConfig {
    #[pyo3(get)]
    pub interest_threshold: f32,
}

#[pymethods]
impl SimpleTrackerConfig {
    #[new]
    pub fn py_new(interest_threshold: f32) -> Self {
        Self { interest_threshold }
    }
}
