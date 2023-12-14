#! /usr/bin/env sh

# kill all background processes when exiting
cleanup() {
  kill 0
}

# trap the SIGINT signal (ctrl-c)
trap cleanup INT

cargo r --bin uttt-server --features auto_connect &
cargo r --bin uttt-client --features "auto_connect auto_play auto_next_round" &
cargo r --bin uttt-client --features "auto_connect auto_play auto_next_round" &

wait
