use pyo3::{exceptions::PyValueError, pyclass, pymethods, types::PyBytes, Bound, PyResult, Python};
use serde::{Deserialize, Serialize};

use super::{Diagnostics, TrackingGraph};

#[pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BeaverFile {
    tracking_graph: Option<TrackingGraph>,
    diagnostics: Option<Diagnostics>,
}

#[pymethods]
impl BeaverFile {
    /// Deserialize a beaver file from bytes.
    #[staticmethod]
    pub fn from_bytes(bytes: &[u8]) -> PyResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|_| PyValueError::new_err("failed to deserialize beaver file"))
    }

    /// Serialize the beaver file to bytes.
    pub fn to_bytes<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyBytes>> {
        let bytes = bincode::serialize(self)
            .map_err(|_| PyValueError::new_err("failed to serialize beaver file"))?;
        Ok(PyBytes::new_bound(py, &bytes))
    }

    pub fn get_tracking_graph(&mut self) -> PyResult<TrackingGraph> {
        if self.tracking_graph.is_some() {
            Ok(std::mem::replace(&mut self.tracking_graph, None).unwrap())
        } else {
            Err(PyValueError::new_err(
                "the beaver file does not contain a tracking graph",
            ))
        }
    }

    pub fn get_diagnostics(&mut self) -> PyResult<Diagnostics> {
        if self.diagnostics.is_some() {
            Ok(std::mem::replace(&mut self.diagnostics, None).unwrap())
        } else {
            Err(PyValueError::new_err(
                "the beaver file does not contain diagnostics",
            ))
        }
    }
}
