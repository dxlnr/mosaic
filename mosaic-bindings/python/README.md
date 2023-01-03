<h1 align="center">
  <b>Python Binding</b><br>
</h1>

The src/ directory contains the binding from Rust to Python. Building the bridge between both
worlds is done by using [Setuptools](https://github.com/PyO3/setuptools-rust) plugin which is able to build and publish Rust Python extensions implemented with [PyO3](https://github.com/PyO3/pyo3) or [rust-cpython](https://github.com/dgrunwald/rust-cpython).

### Building

```shell
# prerequisits
conda activate modalic && pip install setuptools-rust
# The .so file will be included into the modalic python sdk library and serve as the bridge.
python setup.py bdist_wheel
```
