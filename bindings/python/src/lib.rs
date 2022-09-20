use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(mosaic_sdk, ClientInit, PyException);

/// Python module created by decorating a Rust function with #[pymodule].
///
#[pymodule]
fn mosaic_sdk(py: Python, m: &PyModule) -> PyResult<()> {
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
    pub fn new(server_address: String) -> PyResult<Self> {
        let conf = client_sdk::Conf::init_from_params(Some(server_address))
            .map_err(|err| ClientInit::new_err(format!("Conf could not be processed: {}", err)))?;

        let inner = client_sdk::Client::init(conf)
            .map_err(|err| ClientInit::new_err(format!("Client Initialization failed: {}", err)))?;

        Ok(Self { inner: Some(inner) })
    }
}
