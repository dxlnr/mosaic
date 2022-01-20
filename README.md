<h1 align="center">
  <b>Mosaic</b><br>
</h1>

MLOps platform designed for the automotive industry.
## Installation
```sh
# Install rust and cargo
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

## Running the Server
```shell
# First time running takes a while to download and compile
cd source/ && cargo run -p server -- ../configs/config.toml
```
