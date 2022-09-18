use pyo3::prelude::*;

#[pyclass]
struct Client {
    inner: Option<client_sdk::Client>,
}

#[pymethods]
impl Client {}