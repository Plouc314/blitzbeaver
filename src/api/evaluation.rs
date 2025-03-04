use pyo3::{pyclass, pyfunction, pymethods};

use crate::evaluation;

use super::TrackingGraph;

/// EvalMetricChainLength
///
/// Metrics of the length of the tracking chains.
#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct EvalMetricChainLength {
    #[pyo3(get)]
    pub average: f32,
    #[pyo3(get)]
    pub median: f32,
    #[pyo3(get)]
    pub max: f32,
    #[pyo3(get)]
    pub min: f32,
    #[pyo3(get)]
    pub histogram: Vec<u32>,
}

#[pymethods]
impl EvalMetricChainLength {
    pub fn __repr__(&self) -> String {
        format!(
            "EvalMetricChainLength(average={}, median={}, max={}, min={}, histogram=[{}])",
            self.average,
            self.median,
            self.max,
            self.min,
            self.histogram
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

/// EvalMetricGraphProperties
///
/// Metrics of the properties of the tracking graph.
#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct EvalMetricGraphProperties {
    /// For each frame, the ratio of nodes (records) that have matched with an existing tracking chain.
    #[pyo3(get)]
    pub records_match_ratios: Vec<f32>,
    /// For each frame, the ratio of trackers that have match with a node (record).
    #[pyo3(get)]
    pub trackers_match_ratios: Vec<f32>,
    /// For each frame, the ratio of nodes (records) that have conflicts, a conflict occurs
    /// when multiple tracking chains match to the same node.
    #[pyo3(get)]
    pub conflict_ratios: Vec<f32>,
}

#[pymethods]
impl EvalMetricGraphProperties {
    pub fn __repr__(&self) -> String {
        format!(
            "EvalMetricGraphProperties(records_match_ratios=[{}], trackers_match_ratios=[{}], conflict_ratios=[{}])",
            self.records_match_ratios
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.trackers_match_ratios
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.conflict_ratios
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[pyfunction]
pub fn evaluate_tracking_chain_length(graph: &TrackingGraph) -> EvalMetricChainLength {
    evaluation::eval_tracking_chain_length(graph)
}

#[pyfunction]
pub fn evaluate_tracking_graph_properties(graph: &TrackingGraph) -> EvalMetricGraphProperties {
    evaluation::eval_tracking_graph_properties(graph)
}
