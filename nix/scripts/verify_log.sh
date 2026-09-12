verify_log_output() {
    set +x
    local expected_file="${1:-outputs/log.txt}"
    local actual_output
    actual_output=$(cat)

    if [ -f "$expected_file" ]; then
        if diff -u "$expected_file" <(echo "$actual_output") > /dev/null; then
            echo "✓ Log output matches $expected_file"
        else
            echo "✗ Log output mismatch for $expected_file"
            diff -u "$expected_file" <(echo "$actual_output") || true
            exit 1
        fi
    else
        echo "✗ Expected output file not found: $expected_file"
        exit 1
    fi
    set -x
}
