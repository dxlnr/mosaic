use pyo3::prelude::*;
use pyo3::create_exception;
use pyo3::exceptions::PyException;

create_exception!(xaynet_sdk, ClientInit, PyException);

/// Python module created by decorating a Rust function with #[pymodule].
/// 
#[pymodule]
fn modalic_client_sdk(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Client>()?;

    m.add("ClientInit", py.get_type::<ClientInit>())?;

    Ok(())
}

/// Python Class managed behind a single decorator.
/// 
#[pyclass]
#[allow(dead_code)]
struct Client {
    inner: Option<client_sdk::Client>,
}

#[pymethods]
impl Client {
    #[new]
    pub fn new() -> PyResult<Self> {

        let inner = client_sdk::Client::init().map_err(|err| {
            ClientInit::new_err(format!("Client Initialization failed: {}", err))
        })?;
        Ok(Self { inner: Some(inner) })
    }
}