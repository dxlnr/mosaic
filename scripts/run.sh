#!/bin/sh

[ ! -z "$CONFIG_TOML" ] && echo "${CONFIG_TOML}" > ./configs/config.toml
cat ./configs/config.toml

./target/release/mosaic -c configs/config.toml
