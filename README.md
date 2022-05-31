<h1 align="center">
  <b>Mosaic</b><br>
</h1>

The *Mosaic* aggregation server is the backbone of the Modalic MLOps platform designed for enabling Federated Learning.
All the aggregation converges at Mosaic which aims for safety, reliability and performance.

## Building

### Build from Source

If the preferred choice is to build from source, first install Rust. PATH environment variable may be needed to add to Cargo's bin directory. Restarting your computer will do this automatically.

```bash
# Install rust and cargo
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

If you already have Rust installed, make sure you're using the latest version by running:

```bash
rustup update
```

**Release Build** the aggregation server application by cloning this repository and running the following commands from the root
directory of the repo:

```bash
cargo build --release
```

This might take a while for the first time. Note that compilation is a memory intensive process. We recommend having 4 GiB of physical RAM or swap available.

## Running the Server

Start the server application by running: 
```bash
./target/release/mosaic -c configs/config.toml
```


## Running MinIO
```shell
# Running MinIO with Docker which will enable the server to fetch and save data to storage.
docker run \
  -p 9000:9000 \
  -p 9001:9001 \
  -e "MINIO_ROOT_USER=modalic" \
  -e "MINIO_ROOT_PASSWORD=12345678" \
  quay.io/minio/minio server /data --console-address ":9001"
```
More information: [MinIO Docker Quickstart Guide](https://docs.min.io/docs/minio-docker-quickstart-guide.html)

## Running the Docs
```shell
cargo doc --open
```

## Open Issues
- Keep an eye on FedAdam as update averaged model is used as xt.
- Implement clean aggregation strategy selection from config file (hardcoded at the moment).
- Find a proper way to restrict the precision for the Rationals.
- Add metadata to the stored objects (especially to global model)
- Establish default values for settings (done) & just include the most important in example .toml (rest will be covered in docu) but maybe without extra toml file but rather as default values in code.
- get rid of code duplication in fedopt algos
- error handling.

## Contributing

## License