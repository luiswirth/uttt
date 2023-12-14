#! /usr/bin/env sh

# kill all background processes when exiting
cleanup() {
  kill 0
}

# trap the SIGINT signal (ctrl-c)
trap cleanup INT

cargo r --features auto_connect --bin uttt-server &
cargo r --features auto_connect --bin uttt-client &
cargo r --features auto_connect --bin uttt-client &

wait
