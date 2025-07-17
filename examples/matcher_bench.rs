use runome::dictionary::{dict::Matcher, loader, DictionaryResource};
use std::path::PathBuf;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load FST for Matcher
    let sysdic_path = PathBuf::from("sysdic");
    if !sysdic_path.exists() {
        eprintln!("Error: sysdic directory not found at {:?}", sysdic_path);
        eprintln!("Please ensure the sysdic directory exists in the project root.");
        std::process::exit(1);
    }

    println!("Loading FST data...");
    let fst_bytes = loader::load_fst_bytes(&sysdic_path)?;
    let matcher = Matcher::new(fst_bytes)?;

    // Test words of varying lengths
    let test_words = vec![
        ("東", "Single character"),
        ("東京", "Two characters"),
        ("東京都", "Three characters"),
        ("すもももももももものうち", "Long hiragana"),
        ("関西国際空港", "Six characters"),
        ("メロスは激怒した", "Mixed characters"),
    ];

    // Warm up
    println!("\nWarming up...");
    for _ in 0..1000 {
        for (word, _) in &test_words {
            let _ = matcher.run(word, true);
            let _ = matcher.run(word, false);
        }
    }

    // Benchmark exact match
    println!("\n=== Exact Match Performance ===");
    for (word, desc) in &test_words {
        let iterations = 100_000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = matcher.run(word, false);
        }
        
        let duration = start.elapsed();
        let per_call = duration / iterations;
        println!(
            "{} ({}): {:.2} ns/call ({} iterations in {:.2} ms)",
            word,
            desc,
            per_call.as_nanos(),
            iterations,
            duration.as_millis()
        );
    }

    // Benchmark prefix match
    println!("\n=== Prefix Match Performance ===");
    for (word, desc) in &test_words {
        let iterations = 100_000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = matcher.run(word, true);
        }
        
        let duration = start.elapsed();
        let per_call = duration / iterations;
        println!(
            "{} ({}): {:.2} ns/call ({} iterations in {:.2} ms)",
            word,
            desc,
            per_call.as_nanos(),
            iterations,
            duration.as_millis()
        );
    }

    // Profile allocation-heavy case
    println!("\n=== Allocation Profile (prefix match on long string) ===");
    let long_word = "これは非常に長い日本語の文章でマッチャーのパフォーマンスをテストするためのものです";
    let iterations = 10_000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _ = matcher.run(long_word, true);
    }
    
    let duration = start.elapsed();
    let per_call = duration / iterations;
    println!(
        "Long string ({}): {:.2} µs/call ({} iterations in {:.2} ms)",
        long_word.chars().count(),
        per_call.as_micros(),
        iterations,
        duration.as_millis()
    );

    Ok(())
}