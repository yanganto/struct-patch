set -ex
cd $(git rev-parse --show-toplevel 2>/dev/null)
cd examples/complex-examples
cargo test --quiet -p substrate
cargo test --quiet -p catalyst
cargo test --quiet -p catalyst-src

echo "Run catalyst test with unsafe features"
cargo test --quiet -p catalyst --features unsafe
cargo test --quiet -p catalyst-src --features unsafe
