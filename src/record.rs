use pyo3::{pyclass, pymethods};
use pyo3_polars::PyDataType;

#[pyclass(frozen)]
pub struct RecordSchema {
    #[pyo3(get)]
    fields: Vec<FieldSchema>,
}

#[pymethods]
impl RecordSchema {
    #[new]
    fn py_new(fields: Vec<FieldSchema>) -> Self {
        Self { fields }
    }

    fn __repr__(&self) -> String {
        format!(
            "RecordSchema(fields=[{}])",
            self.fields
                .iter()
                .map(|f| f.__repr__())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct FieldSchema {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    dtype: PyDataType,
}

#[pymethods]
impl FieldSchema {
    #[new]
    fn py_new(name: String, dtype: PyDataType) -> Self {
        Self { name, dtype }
    }

    fn __repr__(&self) -> String {
        format!("FieldSchema(name={}, dtype={})", &self.name, &self.dtype.0)
    }
}
