#! /usr/bin/env sh

server_features="auto_connect"
client0_features="auto_connect auto_play auto_next_round"
client1_features=$client0_features

# kill all background processes when exiting
cleanup() {
  kill 0
}

# trap the SIGINT signal (Ctrl-C)
trap cleanup INT

cargo r --release --bin uttt-server --features "$server_features" &
cargo r --release --bin uttt-client --features "$client0_features" &
cargo r --release --bin uttt-client --features "$client1_features" &

wait
