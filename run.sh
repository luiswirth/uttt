#! /usr/bin/env sh

cargo r --features auto_connect --bin uttt-server &
cargo r --features auto_connect --bin uttt-client &
cargo r --features auto_connect --bin uttt-client
