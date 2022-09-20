use pyo3::prelude::*;
use pyo3::create_exception;
use pyo3::exceptions::PyException;

create_exception!(xaynet_sdk, ClientInit, PyException);
create_exception!(xaynet_sdk, ParticipantInit, PyException);

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
    pub fn new(py: Python) -> PyResult<Self> {

        let conf = client_sdk::Conf::init_from_path(None).map_err(|err| {
            ClientInit::new_err(format!("Conf could not be read: {}", err))
        })?;

        let inner = client_sdk::Client::init(conf).map_err(|err| {
            ClientInit::new_err(format!("Client Initialization failed: {}", err))
        })?;

        Ok(Self { inner: Some(inner) })

    }
}