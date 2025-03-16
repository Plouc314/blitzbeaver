use std::collections::HashMap;

use pyo3::pyclass;
use serde::{Deserialize, Serialize};

use crate::id::ID;

#[pyclass(frozen)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackerRecordDiagnostics {
    #[pyo3(get)]
    pub record_idx: usize,
    #[pyo3(get)]
    pub record_score: f32,
    #[pyo3(get)]
    pub distances: Vec<Option<f32>>,
}

impl TrackerRecordDiagnostics {
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
pub struct TrackerFrameDiagnostics {
    #[pyo3(get)]
    pub frame_idx: usize,
    #[pyo3(get)]
    pub records: Vec<TrackerRecordDiagnostics>,
    #[pyo3(get)]
    pub memory: Vec<Vec<String>>,
}

impl TrackerFrameDiagnostics {
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
    pub frames: Vec<TrackerFrameDiagnostics>,
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
pub struct ResolvingDiagnostics {
    #[pyo3(get)]
    pub histogram_record_matchs: Vec<usize>,
    #[pyo3(get)]
    pub histogram_tracker_matchs: Vec<usize>,
}

impl ResolvingDiagnostics {
    pub fn new() -> Self {
        Self {
            histogram_record_matchs: Vec::new(),
            histogram_tracker_matchs: Vec::new(),
        }
    }
}

#[pyclass(frozen)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Diagnostics {
    #[pyo3(get)]
    pub trackers: HashMap<ID, TrackerDiagnostics>,
    #[pyo3(get)]
    pub resolvings: Vec<ResolvingDiagnostics>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            trackers: HashMap::new(),
            resolvings: Vec::new(),
        }
    }
}
