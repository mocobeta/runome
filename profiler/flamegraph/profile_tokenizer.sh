#!/bin/bash

# Script to profile Tokenizer::tokenize() and generate flamegraph
# Requires: perf, cargo-flamegraph

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/output"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "Building release binary with debug symbols..."
cd "$PROJECT_ROOT"
cargo build --release --example tokenizer_bench

echo "Recording performance data..."
# Run perf record on the tokenizer benchmark
sudo perf record -F 99 -a -g -- cargo run --release --example tokenizer_bench

echo "Generating flamegraph..."
# Generate flamegraph using cargo-flamegraph
cargo flamegraph --bin tokenizer_bench --output "$OUTPUT_DIR/tokenizer_flamegraph.svg"

echo "Flamegraph generated at: $OUTPUT_DIR/tokenizer_flamegraph.svg"
echo "You can view it in a web browser."