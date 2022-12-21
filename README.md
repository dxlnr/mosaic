![Modalic Logo](https://github.com/modalic/mosaic/blob/main/public/mo-logo.png)

--------------------------------------------------------------------------------

<h1 align="center">
  <b>Mosaic: Aggregation Server</b><br>
</h1>

<p align="center">
    <a href="https://www.rust-lang.org/">
      <img src="https://img.shields.io/badge/Rust-1.62.1-2F54D1.svg" /></a>
    <a href="https://github.com/modalic/mosaic/blob/main/LICENSE">
      <img src="https://img.shields.io/badge/license-apache2-351c75.svg" /></a>
    <a href="https://github.com/modalic/mosaic/blob/main/CONTRIBUTING.md">
      <img src="https://img.shields.io/badge/PRs-welcome-6834D5.svg" /></a>
</p>

The *Mosaic* aggregation server is the backbone of the Modalic FL Operations Platform designed for enabling Federated Learning in production. The server has three main components: Coordinator, Selector, and Aggregator and is influenced by [Papaya: Practical, Private, and Scalable Federated Learning](resources/Papaya%3A%20Practical%20private%20%26%20scalable%20Federated%20Learning.pdf).
All the aggregation converges at **mosaic** which aims for safety, reliability and performance and is implemented in the *mosaic* directory. 

## Usage

## Building

#### Build from Source

If the preferred choice is to build from source, first install Rust. PATH environment variable may be needed to add to Cargo's bin directory. Restarting your computer will do this automatically.

```bash
# Install rust and cargo
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

If you already have Rust installed, make sure you're using the latest version by running:

```bash
rustup update
```

**Release Build** the aggregation server application by cloning this repository and running the following commands from the *mosaic/* directory of the repository:

```bash
cd mosaic && cargo build --release
```

This might take a while for the first time. Note that compilation is a memory intensive process. We recommend having 4 GiB of physical RAM or swap available.

#### Running the Docs
```shell
cargo doc --open
```

## Running the Server
The server can be run with *default* parameters just by not providing any *.toml* configuration file. 
In order to set important parameters use **-c ${configPATH}**:
```toml
# -c configs/config.toml

# REST API settings.
[api]
# The address to which the REST API of the server
# will be bound. All requests should be sent to this address.
server_address = "127.0.0.1:8080"

# Hyperparameter controlling the Federated Learning training process.
[protocol]
# Defines the number of training rounds (global epochs) 
# that will be performed.
training_rounds = 10
# Sets the number of participants & local models 
# one global epoch should at least contain.
participants = 2
```

Start the server application by running:
```bash
./mosaic/target/release/mosaic -c configs/config.toml
```


## Running MinIO
```shell
# Running MinIO with Docker which will enable the server to fetch and save data to storage.
docker run \
  -p 9000:9000 \
  -p 9001:9001 \
  -e "MINIO_ROOT_USER=modalic" \
  -e "MINIO_ROOT_PASSWORD=" \
  quay.io/minio/minio server /data --console-address ":9001"
```
More information: [MinIO Docker Quickstart Guide](https://docs.min.io/docs/minio-docker-quickstart-guide.html)

## Contributing

## License

The Mosaic Aggregation Module is distributed under the terms of the Apache License Version 2.0. A complete version of the license is available in [LICENSE](LICENSE).