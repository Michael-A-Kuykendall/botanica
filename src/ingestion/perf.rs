/// Search performance benchmarking
use sqlx::SqlitePool;
use std::time::Instant;

pub async fn benchmark_search(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    // 10 representative queries
    let queries = vec![
        "rosa",
        "quercus",
        "pinus",
        "acer",
        "tulip",
        "orchid",
        "fern",
        "moss",
        "herb",
        "oak",
    ];

    let mut latencies = Vec::new();

    println!("Running search benchmarks ({} queries)...", queries.len());
    for q in queries {
        let start = Instant::now();
        let _results: Vec<(String,)> = sqlx::query_as(
            "SELECT scientific_name FROM species_name_fts WHERE scientific_name MATCH ?1 LIMIT 50"
        )
        .bind(q)
        .fetch_all(pool)
        .await?;
        let elapsed = start.elapsed().as_millis();
        latencies.push(elapsed);
        println!("  '{}': {} ms", q, elapsed);
    }

    // Stats
    latencies.sort();
    let mean = latencies.iter().sum::<u128>() / latencies.len() as u128;
    let median = latencies[latencies.len() / 2];
    let p95 = latencies[(latencies.len() * 95) / 100];

    println!("\nStats:");
    println!("  Mean: {} ms", mean);
    println!("  Median: {} ms", median);
    println!("  P95: {} ms", p95);

    if p95 <= 200 {
        println!("✓ PASS: P95 latency {} ms <= 200 ms", p95);
    } else {
        println!("✗ FAIL: P95 latency {} ms > 200 ms", p95);
    }

    Ok(())
}
