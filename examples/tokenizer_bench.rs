use runome::tokenizer::Tokenizer;
use std::env;
use std::fs;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    let bench_mode = args.iter().any(|arg| arg == "--bench");

    // Initialize tokenizer
    let tokenizer = match Tokenizer::new(None, None) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to initialize tokenizer: {}", e);
            std::process::exit(1);
        }
    };

    // Load test text
    let text = if let Ok(content) = fs::read_to_string("tests/text_lemon.txt") {
        content
    } else {
        // Fallback text if file not found
        "これは日本語のテスト文章です。形態素解析を行います。".to_string()
    };

    if bench_mode {
        // Benchmark mode - run multiple iterations for profiling
        let iterations = 1;
        let mut total_tokens = 0;

        eprintln!("Running benchmark with {} iterations...", iterations);
        let start = Instant::now();

        for _ in 0..iterations {
            let tokens: Vec<_> = tokenizer
                .tokenize(&text, None, None)
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            total_tokens += tokens.len();
        }

        let duration = start.elapsed();
        eprintln!("Processed {} tokens in {:?}", total_tokens, duration);
        eprintln!("Average time per iteration: {:?}", duration / iterations);
    } else {
        // Normal mode - single run with output
        let tokens: Vec<_> = tokenizer
            .tokenize(&text, None, None)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // Print first 10 tokens
        for (i, token) in tokens.iter().take(10).enumerate() {
            println!("{}: {:?}", i, token);
        }
        println!("... ({} total tokens)", tokens.len());
    }
}
