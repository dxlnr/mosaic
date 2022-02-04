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