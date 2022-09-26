use pyo3::{
    create_exception,
    exceptions::PyException,
    prelude::*,
    types::{PyList, PyLong},
};

use mosaic_core::model::{
    tensor::{FromPrimitives, Tensor, TensorStorage},
    Model,
};

create_exception!(mosaic_sdk, ClientInit, PyException);
create_exception!(mosaic_sdk, ClientNotFound, PyException);
create_exception!(xaynet_sdk, ModelError, PyException);
create_exception!(xaynet_sdk, TensorsError, PyException);
create_exception!(xaynet_sdk, TensorDataTypeMismatch, PyException);
create_exception!(xaynet_sdk, TensorDataTypeError, PyException);
create_exception!(xaynet_sdk, TensorShapeError, PyException);

/// Python module created by decorating a Rust function with #[pymodule].
///
#[pymodule]
fn mosaic_sdk(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_class::<MosaicTensor>()?;

    m.add("ClientInit", py.get_type::<ClientInit>())?;
    m.add("ClientNotFound", py.get_type::<ClientNotFound>())?;
    m.add("ModelError", py.get_type::<ModelError>())?;
    m.add("TensorsError", py.get_type::<TensorsError>())?;
    m.add(
        "TensorDataTypeMismatch",
        py.get_type::<TensorDataTypeMismatch>(),
    )?;
    m.add("TensorDataTypeError", py.get_type::<TensorDataTypeError>())?;
    m.add("TensorShapeError", py.get_type::<TensorShapeError>())?;

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

    pub fn set_model(&mut self, tensors: &PyList, model_version: &PyLong) -> PyResult<()> {
        let inner = match self.inner {
            Some(ref mut inner) => inner,
            None => {
                return Err(ClientNotFound::new_err(
                    "Client not found. Unable to perform task.",
                ))
            }
        };
        let mosaic_tensors: Vec<MosaicTensor> = tensors
            .extract()
            .map_err(|err| TensorsError::new_err(format!("{}", err)))?;
        let model_tensors = mosaic_tensors
            .iter()
            .map(|mt| mt.inner.to_owned().unwrap())
            .collect();

        let model_version = model_version
            .extract()
            .map_err(|err| ModelError::new_err(format!("{}", err)))?;

        let model = Model::new(model_tensors, model_version);
        inner.set_model(model);
        Ok(())
    }
}

/// Python Class managed behind a single decorator.
///
#[pyclass]
#[allow(dead_code)]
struct MosaicTensor {
    inner: Option<Tensor>,
}

#[pymethods]
impl MosaicTensor {
    #[new]
    pub fn new(storage: &PyList, dtype: &PyLong, shape: &PyList) -> PyResult<Self> {
        let tensor_storage = from_primitives!(storage, f32);
        let tensor_shape: Vec<i32> = shape
            .extract()
            .map_err(|err| TensorShapeError::new_err(format!("{}", err)))?;
        let tensor_dtype: i32 = dtype
            .extract()
            .map_err(|err| TensorDataTypeError::new_err(format!("{}", err)))?;

        let inner = Tensor::init(tensor_storage, tensor_dtype, tensor_shape);
        Ok(Self { inner: Some(inner) })
    }
}

impl<'source> FromPyObject<'source> for MosaicTensor {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let motens = ob.extract()?;
        Ok(motens)
    }
}

#[macro_export]
macro_rules! from_primitives {
    ($tensor_storage:expr, $data_type:ty) => {{
        let tensor_storage: Vec<$data_type> = $tensor_storage
            .extract()
            .map_err(|err| TensorDataTypeMismatch::new_err(format!("{}", err)))?;
        TensorStorage::from_primitives(tensor_storage.into_iter())
    }};
}
