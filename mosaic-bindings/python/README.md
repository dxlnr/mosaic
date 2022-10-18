<h1 align="center">
  <b>Python Binding</b><br>
</h1>

The src/ directory contains the binding from Rust to Python. Building the bridge between both
worlds is done by using [Maturin](https://github.com/PyO3/maturin) which is able to build and publish
crates with pyo3, rust-cpython and cffi bindings as well as rust binaries as python packages.

### Building

```shell
# prerequisits
conda activate modalic && pip install maturin
# Maturin build will create a wheel which is present in target/wheel/ folder.
#
# The .so file will be included into the modalic python sdk library and serve as the bridge.
maturin build
```
