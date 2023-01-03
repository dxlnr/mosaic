from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="mosaic_python_sdk",
    version="0.2.0",
    rust_extensions=[RustExtension("mosaic_python_sdk.mosaic_python_sdk", binding=Binding.PyO3)],
    packages=["mosaic_python_sdk"],
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
)
