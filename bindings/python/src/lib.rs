use pyo3::{create_exception, exceptions::PyException, prelude::*};



create_exception!(mosaic_sdk, ClientInit, PyException);
create_exception!(mosaic_sdk, ClientNotFound, PyException);

/// Python module created by decorating a Rust function with #[pymodule].
///
#[pymodule]
fn mosaic_sdk(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Client>()?;

    m.add("ClientInit", py.get_type::<ClientInit>())?;
    m.add("ClientNotFound", py.get_type::<ClientNotFound>())?;

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
        println!("  (1) Starting to create Rust Client in Python.");
        let conf = client_sdk::Conf::init_from_params(Some(server_address))
            .map_err(|err| ClientInit::new_err(format!("Conf could not be processed: {}", err)))?;
        println!("  (2) Read config file.");
        let inner = client_sdk::Client::init(conf)
            .map_err(|err| ClientInit::new_err(format!("Client Initialization failed: {}", err)))?;
        println!("  (3) Rust Client is ready for usage.");
        Ok(Self { inner: Some(inner) })
    }

    pub fn task(&self) -> PyResult<u8> {
        let inner = match self.inner {
            Some(ref inner) => inner,
            None => {
                return Err(ClientNotFound::new_err(
                    "Client not found. Unable to perform task.",
                ))
            }
        };

        // FIXME:
        // Returning an enum is currently not supported: https://github.com/PyO3/pyo3/pull/1045
        let task_as_u8 = match inner.task() {
            client_sdk::Task::None => 0,
            client_sdk::Task::Connect => 1,
            client_sdk::Task::Update => 2,
        };

        Ok(task_as_u8)
    }

    pub fn step(&mut self) -> PyResult<()> {
        let inner = match self.inner {
            Some(ref mut inner) => inner,
            None => {
                return Err(ClientNotFound::new_err(
                    "Client not found. Unable to perform task.",
                ))
            }
        };

        inner.step();
        Ok(())
    }


}
