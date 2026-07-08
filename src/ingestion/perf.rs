/// Search performance benchmarking
use std::time::Instant;
use crate::database::BotanicalDatabase;

pub async fn benchmark_search(db: &BotanicalDatabase) -> Result<(), Box<dyn std::error::Error>> {
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
        let pattern = format!("%{}%", q);
        let conn = db.conn().await;
        let mut stmt = conn.prepare("SELECT id FROM species WHERE specific_epithet ILIKE ? LIMIT 50")?;
        let _rows: Vec<String> = stmt
            .query_map([&pattern], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);
        drop(conn);
        let elapsed = start.elapsed().as_millis();
        latencies.push(elapsed);
        println!("  '{}': {} ms", q, elapsed);
    }

    latencies.sort();
    let mean = latencies.iter().sum::<u128>() / latencies.len() as u128;
    let median = latencies[latencies.len() / 2];
    let p95 = latencies[(latencies.len() * 95) / 100];

    println!("\nStats:");
    println!("  Mean: {} ms", mean);
    println!("  Median: {} ms", median);
    println!("  P95: {} ms", p95);

    if p95 <= 200 {
        println!("PASS: P95 latency {} ms <= 200 ms", p95);
    } else {
        println!("FAIL: P95 latency {} ms > 200 ms", p95);
    }

    Ok(())
}
