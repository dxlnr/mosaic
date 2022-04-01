#!/bin/sh

[ ! -z "$CONFIG_TOML" ] && echo "${CONFIG_TOML}" > ./configs/config.toml
cat ./configs/config.toml

cargo run -p mosaic --release -- -c configs/config.toml
