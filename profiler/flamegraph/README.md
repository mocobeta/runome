# Flamegraph Profiling for Tokenizer

This directory contains scripts to generate flamegraph visualizations of the `Tokenizer::tokenize()` method's stack traces.

## Prerequisites

1. Install cargo-flamegraph:
   ```bash
   cargo install flamegraph
   ```

2. On Linux, you may need perf tools:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install linux-tools-common linux-tools-generic linux-tools-`uname -r`
   
   # Fedora/RHEL
   sudo dnf install perf
   ```

3. Ensure you have permissions to use perf (may require sudo or adjusting kernel parameters).

## Usage

### Method 1: Using cargo-flamegraph directly

Run the flamegraph script:
```bash
./flamegraph_tokenizer.sh
```

This will:
- Build the tokenizer benchmark in release mode with debug symbols
- Profile the execution using cargo-flamegraph
- Generate a flamegraph SVG in `output/tokenizer_flamegraph.svg`
- Optionally create a perf-based flamegraph if perf is available

### Method 2: Manual profiling

1. Build the example with debug symbols:
   ```bash
   CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release --example tokenizer_bench
   ```

2. Generate the flamegraph:
   ```bash
   cargo flamegraph --example tokenizer_bench -o output/manual_flamegraph.svg -- --bench
   ```

## Viewing Results

Open the generated SVG files in a web browser:
```bash
firefox output/tokenizer_flamegraph.svg
# or
chrome output/tokenizer_flamegraph.svg
```

## Understanding the Flamegraph

- **Width**: Represents the percentage of samples where the function was on the stack
- **Height**: Shows the call stack depth
- **Colors**: Usually warm colors (red/orange) indicate hot paths
- **Interactive**: Click on functions to zoom in, reset zoom with "Reset Zoom" button

## Performance Notes

- The benchmark runs only 1 iteration by default to keep profiling time reasonable
- For more accurate profiling, edit `examples/tokenizer_bench.rs` and increase iterations
- The test file `tests/text_lemon.txt` contains Japanese text from "Lemon" by Motojirō Kajii

## Troubleshooting

1. **Permission denied for perf**:
   ```bash
   echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
   ```

2. **Missing debug symbols**: Ensure you're building with `CARGO_PROFILE_RELEASE_DEBUG=true`

3. **Flamegraph not found**: Install with `cargo install flamegraph`