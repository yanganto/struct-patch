cargo login $1
cargo publish -p struct-patch-derive || echo "publish struct-patch-derive fail"
sleep 10
cargo publish -p struct-patch
