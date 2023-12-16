#! /usr/bin/env sh

# feature flags
auto_connect=true
auto_play=true
auto_next_round=true

server_features=""
client_features=""

[ $auto_connect = true ] && server_features="${server_features}auto_connect " && client_features="${client_features}auto_connect "
[ $auto_play = true ] && client_features="${client_features}auto_play "
[ $auto_next_round = true ] && client_features="${client_features}auto_next_round "

cargo_cmd="cargo r"

if [ "$1" = "--release" ]; then
  cargo_cmd="cargo run --release"
fi

# kill all processes when exiting
cleanup() {
  kill 0
}
# trap the SIGINT signal (Ctrl-C)
trap cleanup INT

$cargo_cmd --bin uttt-server --features "$server_features" &
$cargo_cmd --bin uttt-client --features "$client_features" &
$cargo_cmd --bin uttt-client --features "$client_features" &

wait
