use pyo3::{pyclass, pymethods};

#[pyclass(eq, eq_int)]
#[derive(PartialEq, Clone, Copy)]
pub enum ElementType {
    String,
    MultiStrings,
}

impl ElementType {
    fn __repr__(&self) -> String {
        String::from(match self {
            Self::String => "String",
            Self::MultiStrings => "MultiStrings",
        })
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct RecordSchema {
    #[pyo3(get)]
    pub fields: Vec<FieldSchema>,
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
    pub name: String,
    #[pyo3(get)]
    pub dtype: ElementType,
}

#[pymethods]
impl FieldSchema {
    #[new]
    fn py_new(name: String, dtype: ElementType) -> Self {
        Self { name, dtype }
    }

    fn __repr__(&self) -> String {
        format!(
            "FieldSchema(name={}, dtype={})",
            &self.name,
            self.dtype.__repr__()
        )
    }
}
