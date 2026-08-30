set -ex
cd $(git rev-parse --show-toplevel 2>/dev/null)
cd examples/patch-examples

run_no_default() {
    cargo run --quiet --no-default-features --example instance
    cargo run --quiet --no-default-features --example diff
    cargo run --quiet --no-default-features --example json
    cargo run --quiet --no-default-features --example rename-patch-struct
    cargo run --quiet --no-default-features --example patch-attr
    cargo run --quiet --no-default-features --example time
    cargo run --quiet --no-default-features --example clap
    cargo run --quiet --no-default-features --features=nesting --example nesting
    cargo run --quiet --no-default-features --features=option --example option
    cargo run --quiet --no-default-features --example log
    cargo run --quiet --no-default-features --example apply-by
    cargo run --quiet --no-default-features --features=box --example box
}

run_std() {
    cargo run --quiet --features=std --example instance
    cargo run --quiet --features=std --example diff
    cargo run --quiet --features=std --example json
    cargo run --quiet --features=std --example rename-patch-struct
    cargo run --quiet --features=std --example patch-attr
    cargo run --quiet --features=std --example option
    cargo run --quiet --features=std --example box
    cargo run --quiet --features=std,nesting --example nesting
}

run_merge() {
    cargo run --quiet --features=option,merge --example option
    cargo run --quiet --features=merge --example op
    cargo run --quiet --features=merge,nesting --example nesting
}

run_option() {
    cargo run --quiet --features=none_as_default --example option
    cargo run --quiet --features=none_as_default,nesting --example nesting
    cargo run --quiet --features=keep_none --example option
    cargo run --quiet --features=keep_none,nesting --example nesting
}

run_default() {
    cargo run --quiet --example status
    cargo run --quiet --example op
    cargo run --quiet --example clap
    cargo run --quiet --features=nesting --example nesting
    cargo run --quiet --features=nesting --example clap
    cargo run --quiet --example log
    cargo run --quiet --example apply-by
    cargo run --quiet --features=box --example box
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
    option)
        run_option
        ;;
    default)
        run_default
        ;;
    *)
        run_no_default
        run_std
        run_merge
        run_option
        run_default
        ;;
esac
