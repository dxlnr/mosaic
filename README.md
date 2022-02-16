<h1 align="center">
  <b>Mosaic</b><br>
</h1>

Mosaic Server which is the backbone of modalic MLOps platform designed for the automotive industry enabling Federated Learning. 
All the aggregation converges at Mosaic which aims for safety, reliance and performance.

It currently implements the basic **FedAvg** algorithm proposed in McMahan *et al.* [Communication-Efficient Learning of Deep Networks from Decentralized Data](https://arxiv.org/abs/1602.05629).

## Installation
```sh
# Install rust and cargo
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

## Running the Server
```shell
# First time running takes a while to download and compile
cargo run -p server -- -c configs/config.toml
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

## Open Issues
- Keep an eye on FedAdam as update averaged model is used as xt. 
- Implement clean aggregation strategy selection from config file (hardcoded at the moment).
- Find a proper way to restrict the precision for the Rationals.