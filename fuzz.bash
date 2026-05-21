#! /bin/bash

set -e

SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
SCRIPT_DIR="$(dirname -- "${SCRIPT_PATH}")"

cd "${SCRIPT_DIR}"

echo "Fuzzing capnpack_full"
cargo +nightly fuzz run capnpack_full -O -j 3 -- -max_total_time=100

echo "Fuzzing unpack_possibly_invalid"
cargo +nightly fuzz run unpack_possibly_invalid -O -j 3 -- -max_total_time=100

echo "Generating code coverage report!"
bash ./code_cov_fuzz_report_generate.bash