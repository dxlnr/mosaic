# BUILD: docker build -t modalic/worker:mosaic .
# RUN: docker run --rm --network mosaic-docker-compose_default docker run --rm --network mosaic-docker-compose_default -e CONFIG_TOML="$(cat ./configs/config.toml)" modalic/worker:mosaic
FROM rust:latest

WORKDIR /usr/src/app
COPY . .
RUN chmod +x ./run.sh
ENV CONFIG_TOML=


RUN rustup component add rustfmt
RUN cargo build --release

CMD ["sh", "-c", "./run.sh"]

