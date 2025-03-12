use std::collections::HashMap;

use pyo3::pyclass;
use serde::{Deserialize, Serialize};

use crate::id::ID;

#[pyclass(frozen)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordScoreDiagnostics {
    pub record_idx: usize,
    pub record_score: f32,
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
    pub frame_idx: usize,
    pub records: Vec<RecordScoreDiagnostics>,
}

impl FrameDiagnostics {
    pub fn new(frame_idx: usize) -> Self {
        Self {
            frame_idx,
            records: Vec::new(),
        }
    }
}

#[pyclass(frozen)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackerDiagnostics {
    pub id: ID,
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
    pub trackers: HashMap<ID, TrackerDiagnostics>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            trackers: HashMap::new(),
        }
    }
}
