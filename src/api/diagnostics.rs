use std::collections::HashMap;

use pyo3::pyclass;
use serde::{Deserialize, Serialize};

use crate::id::ID;

#[pyclass(frozen)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordScoreDiagnostics {
    #[pyo3(get)]
    pub record_idx: usize,
    #[pyo3(get)]
    pub record_score: f32,
    #[pyo3(get)]
    pub distances: Vec<Option<f32>>,
}

impl RecordScoreDiagnostics {
    pub fn new(record_idx: usize, record_score: f32, distances: Vec<Option<f32>>) -> Self {
        Self {
            record_idx,
            record_score,
            distances,
        }
    }
}

#[pyclass(frozen)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrameDiagnostics {
    #[pyo3(get)]
    pub frame_idx: usize,
    #[pyo3(get)]
    pub records: Vec<RecordScoreDiagnostics>,
    #[pyo3(get)]
    pub memory: Vec<Vec<String>>,
}

impl FrameDiagnostics {
    pub fn new(frame_idx: usize) -> Self {
        Self {
            frame_idx,
            records: Vec::new(),
            memory: Vec::new(),
        }
    }
}

#[pyclass(frozen)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackerDiagnostics {
    #[pyo3(get)]
    pub id: ID,
    #[pyo3(get)]
    pub frames: Vec<FrameDiagnostics>,
}

impl TrackerDiagnostics {
    pub fn new(id: ID) -> Self {
        Self {
            id,
            frames: Vec::new(),
        }
    }
}

#[pyclass(frozen)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Diagnostics {
    #[pyo3(get)]
    pub trackers: HashMap<ID, TrackerDiagnostics>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            trackers: HashMap::new(),
        }
    }
}
