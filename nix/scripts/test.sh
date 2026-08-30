set -ex
cd $(git rev-parse --show-toplevel 2>/dev/null)

run_no_default() {
    cargo test --quiet --no-default-features
}

run_std() {
    cargo test --quiet --features=std
}

run_merge() {
    cargo test --quiet --features=merge --no-default-features
    cargo test --quiet --features=merge
}

run_default() {
    cargo test --quiet
}

case "${1:-}" in
    no-default)
        run_no_default
        ;;
    std)
        run_std
        ;;
    merge)
        run_merge
        ;;
    default)
        run_default
        ;;
    *)
        run_no_default
        run_std
        run_merge
        run_default
        ;;
esac
