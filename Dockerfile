# BUILD: docker build -t modalic/worker:mosaic .
# RUN: docker run --rm --network mosaic-docker-compose_default -v /Users/lukasbraunstorfer/Documents/Projects/Federated/Mosaic/mosaic/configs/config_docker.toml:/usr/src/app/configs/config.toml modalic/worker:mosaic
FROM rust:latest

WORKDIR /usr/src/app
COPY . .

RUN rustup component add rustfmt

CMD cargo run -p server -- -c configs/config.toml