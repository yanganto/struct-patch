set -ex
cd $(git rev-parse --show-toplevel 2>/dev/null)
cd examples/no-std-examples
cargo run --quiet --features=box --bin no-std-box
cargo run --quiet --features=option --bin no-std-option
