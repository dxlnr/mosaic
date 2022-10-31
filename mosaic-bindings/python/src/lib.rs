use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::types::PyList;
use pyo3::{prelude::*, wrap_pyfunction};
use tracing::{debug, info};
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::*;

use mosaic_client_sdk::settings::MaxMessageSize;
use mosaic_core::model::{DataType, FromPrimitives, IntoPrimitives, Model};

create_exception!(mosaic_python_sdk, CryptoInit, PyException);
create_exception!(mosaic_python_sdk, ClientInit, PyException);
create_exception!(mosaic_python_sdk, ClientRestore, PyException);
create_exception!(mosaic_python_sdk, UninitializedClient, PyException);
create_exception!(mosaic_python_sdk, LocalModelLengthMisMatch, PyException);
create_exception!(mosaic_python_sdk, LocalModelDataTypeError, PyException);
create_exception!(mosaic_python_sdk, GlobalModelUnavailable, PyException);
create_exception!(mosaic_python_sdk, GlobalModelDataTypeMisMatch, PyException);

#[pymodule]
fn mosaic_python_sdk(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_function(wrap_pyfunction!(init_logging, m)?)?;

    m.add("CryptoInit", py.get_type::<CryptoInit>())?;
    m.add("ClientInit", py.get_type::<ClientInit>())?;
    m.add("ClientRestore", py.get_type::<ClientRestore>())?;
    m.add("UninitializedClient", py.get_type::<UninitializedClient>())?;
    m.add(
        "LocalModelLengthMisMatch",
        py.get_type::<LocalModelLengthMisMatch>(),
    )?;
    m.add(
        "LocalModelDataTypeError",
        py.get_type::<LocalModelDataTypeError>(),
    )?;
    m.add(
        "GlobalModelUnavailable",
        py.get_type::<GlobalModelUnavailable>(),
    )?;
    m.add(
        "GlobalModelDataTypeMisMatch",
        py.get_type::<GlobalModelDataTypeMisMatch>(),
    )?;

    Ok(())
}

#[pyclass]
// #[text_signature = "(url, scalar, /)"]
struct Client {
    inner: Option<mosaic_client_sdk::Client>,
}

#[pymethods]
impl Client {
    #[new]
    pub fn new(url: String, scalar: f64, state: Option<Vec<u8>>) -> PyResult<Self> {
        sodiumoxide::init()
            .map_err(|_| CryptoInit::new_err("failed to initialize crypto library"))?;

        let inner = if let Some(state) = state {
            debug!("restore client");
            mosaic_client_sdk::Client::restore(&state, &url).map_err(|err| {
                ClientRestore::new_err(format!("failed to restore client: {}", err))
            })?
        } else {
            debug!("initialize client");
            let mut settings = mosaic_client_sdk::Settings::new();
            settings.set_url(url);
            settings.set_keys(mosaic_core::crypto::SigningKeyPair::generate());
            settings.set_scalar(scalar);
            settings.set_max_message_size(MaxMessageSize::unlimited());

            mosaic_client_sdk::Client::new(settings).map_err(|err| {
                ClientInit::new_err(format!("failed to initialize client: {}", err))
            })?
        };

        Ok(Self { inner: Some(inner) })
    }

    // #[text_signature = "($self)"]
    pub fn step(&mut self) -> PyResult<()> {
        let inner = match self.inner {
            Some(ref mut inner) => inner,
            None => {
                return Err(UninitializedClient::new_err(
                    "called 'tick' on an uninitialized client. this is a bug.",
                ))
            }
        };

        inner.step();
        Ok(())
    }

    // #[text_signature = "($self, local_model)"]
    pub fn set_model(&mut self, local_model: &PyList) -> PyResult<()> {
        let inner = match self.inner {
            Some(ref mut inner) => inner,
            None => {
                return Err(UninitializedClient::new_err(
                    "called 'set_model' on an uninitialized client. this is a bug.",
                ))
            }
        };

        let local_model_config = inner.local_model_config();

        // if local_model.len() != local_model_config.len {
        //     return Err(LocalModelLengthMisMatch::new_err(format!(
        //         "the local model length is incompatible with the model length of the current model configuration {} != {}",
        //         local_model.len(),
        //         local_model_config.len
        //     )));
        // }

        debug!(
            "convert local model to {:?} datatype",
            local_model_config.data_type
        );

        match local_model_config.data_type {
            DataType::F32 => from_primitives!(inner, local_model, f32),
            DataType::F64 => from_primitives!(inner, local_model, f64),
            DataType::I32 => from_primitives!(inner, local_model, i32),
            DataType::I64 => from_primitives!(inner, local_model, i64),
        }
    }

    /// Check whether the client internal state machine made progress while
    /// executing the PET protocol. If so, the client state likely changed.
    // #[text_signature = "($self)"]
    pub fn made_progress(&self) -> PyResult<bool> {
        let inner = match self.inner {
            Some(ref inner) => inner,
            None => {
                return Err(UninitializedClient::new_err(
                    "called 'made_progress' on an uninitialized client. this is a bug.",
                ))
            }
        };

        Ok(inner.made_progress())
    }

    /// Check whether the client internal state machine is waiting for the
    /// client to load its model into the store. If this method returns `true`, the
    /// caller should make sure to call [`Client::set_model()`] at some point.
    // #[text_signature = "($self)"]
    pub fn should_set_model(&self) -> PyResult<bool> {
        let inner = match self.inner {
            Some(ref inner) => inner,
            None => {
                return Err(UninitializedClient::new_err(
                    "called 'should_set_model' on an uninitialized client. this is a bug.",
                ))
            }
        };

        Ok(inner.should_set_model())
    }

    // #[text_signature = "($self)"]
    pub fn task(&self) -> PyResult<u8> {
        let inner = match self.inner {
            Some(ref inner) => inner,
            None => {
                return Err(UninitializedClient::new_err(
                    "called 'task' on an uninitialized client. this is a bug.",
                ))
            }
        };

        // FIXME:
        // Returning an enum is currently not supported: https://github.com/PyO3/pyo3/pull/1045
        let task_as_u8 = match inner.task() {
            mosaic_client_sdk::Task::None => 0,
            mosaic_client_sdk::Task::Sum => 1,
            mosaic_client_sdk::Task::Update => 2,
        };

        Ok(task_as_u8)
    }

    // #[text_signature = "($self)"]
    pub fn new_global_model(&self) -> PyResult<bool> {
        let inner = match self.inner {
            Some(ref inner) => inner,
            None => {
                return Err(UninitializedClient::new_err(
                    "called 'new_global_model' on an uninitialized client. this is a bug.",
                ))
            }
        };

        Ok(inner.new_global_model())
    }

    // #[text_signature = "($self)"]
    pub fn global_model(&mut self, py: Python) -> PyResult<Option<Py<PyList>>> {
        let inner = match self.inner {
            Some(ref mut inner) => inner,
            None => {
                return Err(UninitializedClient::new_err(
                    "called 'global_model' on an uninitialized client. this is a bug.",
                ))
            }
        };

        let global_model = inner
            .global_model()
            .map_err(|_| GlobalModelUnavailable::new_err("failed to fetch global model"))?;

        let global_model = match global_model {
            Some(global_model) => global_model,
            None => return Ok(None),
        };

        match inner.local_model_config().data_type {
            DataType::F32 => into_primitives!(py, global_model, f32),
            DataType::F64 => into_primitives!(py, global_model, f64),
            DataType::I32 => into_primitives!(py, global_model, i32),
            DataType::I64 => into_primitives!(py, global_model, i64),
        }
    }

    // #[text_signature = "($self)"]
    pub fn save(&mut self) -> PyResult<Vec<u8>> {
        let inner = match self.inner.take() {
            Some(inner) => inner,
            None => {
                return Err(UninitializedClient::new_err(
                    "called 'save' on an uninitialized client. this is a bug.",
                ))
            }
        };

        Ok(inner.save())
    }
}

#[macro_export]
macro_rules! into_primitives {
    ($py:expr, $global_model:expr, $data_type:ty) => {
        if let Ok(global_model) = $global_model
            .into_primitives()
            .collect::<Result<Vec<$data_type>, _>>()
        {
            let py_list = PyList::new($py, global_model.into_iter());
            Ok(Some(py_list.into()))
        } else {
            Err(GlobalModelDataTypeMisMatch::new_err(
                "the global model data type is incompatible with the data type of the current model configuration",
            ))
        }
    };
}

#[macro_export]
macro_rules! from_primitives {
    ($client:expr, $local_model:expr, $data_type:ty) => {{
            let model: Vec<$data_type> = $local_model.extract()
                .map_err(|err| LocalModelDataTypeError::new_err(format!("{}", err)))?;
            let converted_model = Model::from_primitives(model.into_iter());
            if let Ok(converted_model) = converted_model {
                $client.set_model(converted_model);
                Ok(())
            } else {
                Err(LocalModelDataTypeError::new_err(
                    "the local model data type is incompatible with the data type of the current model configuration"
                ))
            }}
    };
}

#[pyfunction]
fn init_logging() {
    let format = fmt::format()
        .with_level(true)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .compact();

    let _fmt_subscriber = FmtSubscriber::builder()
        .event_format(format)
        .with_env_filter("modalic=debug,info")
        .with_ansi(true)
        .init();
}
