#!/bin/bash

# Script to generate flamegraph for Tokenizer::tokenize() using cargo-flamegraph
# Requires: cargo-flamegraph (install with: cargo install flamegraph)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/output"

# Create output directory
mkdir -p "$OUTPUT_DIR"

cd "$PROJECT_ROOT"

# Check if cargo-flamegraph is installed
if ! command -v cargo-flamegraph &> /dev/null; then
    echo "cargo-flamegraph not found. Installing..."
    cargo install flamegraph
fi

echo "Building and profiling tokenizer example..."

# Generate flamegraph with cargo-flamegraph
# This will build in release mode with debug symbols and profile the execution
cargo flamegraph --example tokenizer_bench -o "$OUTPUT_DIR/tokenizer_flamegraph.svg" -- --bench

echo "Flamegraph saved to: $OUTPUT_DIR/tokenizer_flamegraph.svg"

# Also generate a perf-based flamegraph if perf is available
if command -v perf &> /dev/null; then
    echo "Generating detailed perf flamegraph..."
    
    # Build with full debug info
    CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release --example tokenizer_bench
    
    # Record with perf
    sudo perf record -F 999 -g --call-graph=dwarf target/release/examples/tokenizer_bench --bench
    
    # Generate flamegraph from perf data
    perf script | cargo flamegraph -- --perfdata /dev/stdin -o "$OUTPUT_DIR/tokenizer_perf_flamegraph.svg"
    
    echo "Perf flamegraph saved to: $OUTPUT_DIR/tokenizer_perf_flamegraph.svg"
fi

echo "Done! View the flamegraphs in your web browser."