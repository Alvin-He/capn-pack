#! /bin/bash

set -e

SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
SCRIPT_DIR="$(dirname -- "${SCRIPT_PATH}")"

cd "${SCRIPT_DIR}"

echo "Generating code coverage reports in ./target !"

cargo +nightly fuzz coverage unpack_possibly_invalid -O 

cargo +nightly cov -- show target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/unpack_possibly_invalid \
    --format=html \
    -instr-profile=./fuzz/coverage/unpack_possibly_invalid/coverage.profdata \
    > ./target/unpack.html

cargo +nightly fuzz coverage capnpack_full -O 

cargo +nightly cov -- show target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/capnpack_full \
    --format=html \
    -instr-profile=./fuzz/coverage/capnpack_full/coverage.profdata \
    > ./target/capnpack_full.html