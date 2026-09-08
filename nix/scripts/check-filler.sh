set -ex
cd $(git rev-parse --show-toplevel 2>/dev/null)
cd examples/filler-examples

run_no_default() {
    cargo run --quiet --no-default-features --example filler
    cargo run --quiet --no-default-features --example log
}

run_default() {
    cargo run --quiet --example filler
    cargo run --quiet --example filler-op
    cargo run --quiet --example log
    cargo run --quiet --features nesting --example log
}

case "${1:-}" in
    no-default)
        run_no_default
        ;;
    default)
        run_default
        ;;
    *)
        run_no_default
        run_default
        ;;
esac
