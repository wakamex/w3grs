use std::{
    env,
    hint::black_box,
    process,
    time::{Duration, Instant},
};

use w3grs::W3GReplay;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let replay_path = args.next().ok_or_else(|| {
        "usage: w3grs-bench <replay.w3g|replay.nwg> [iterations] [warmup]".to_string()
    })?;
    let iterations = parse_count(args.next(), 25, "iterations")?;
    let warmup = parse_count(args.next(), 5, "warmup")?;
    let bytes = std::fs::read(&replay_path)
        .map_err(|error| format!("failed to read {replay_path}: {error}"))?;
    let mut parser = W3GReplay::new();

    for _ in 0..warmup {
        let output = parser
            .parse_bytes(black_box(&bytes))
            .map_err(|error| format!("warmup parse failed: {error}"))?;
        black_box(output.players.len());
    }

    let mut samples = Vec::with_capacity(iterations);
    let mut last_players = 0usize;
    for _ in 0..iterations {
        let started = Instant::now();
        let output = parser
            .parse_bytes(black_box(&bytes))
            .map_err(|error| format!("timed parse failed: {error}"))?;
        let elapsed = started.elapsed();
        last_players = output.players.len();
        black_box(&output);
        samples.push(elapsed);
    }

    let stats = Stats::from_samples(&samples);
    println!(
        "{{\"parser\":\"w3grs\",\"iterations\":{iterations},\"warmup\":{warmup},\"totalMs\":{:.6},\"meanMs\":{:.6},\"minMs\":{:.6},\"maxMs\":{:.6},\"lastPlayers\":{last_players}}}",
        stats.total_ms(),
        stats.mean_ms(),
        stats.min_ms(),
        stats.max_ms(),
    );
    Ok(())
}

fn parse_count(value: Option<String>, default: usize, name: &str) -> Result<usize, String> {
    match value {
        Some(value) => value
            .parse::<usize>()
            .map_err(|error| format!("invalid {name} value {value:?}: {error}"))
            .and_then(|value| {
                if value == 0 {
                    Err(format!("{name} must be greater than zero"))
                } else {
                    Ok(value)
                }
            }),
        None => Ok(default),
    }
}

struct Stats {
    total: Duration,
    min: Duration,
    max: Duration,
    count: usize,
}

impl Stats {
    fn from_samples(samples: &[Duration]) -> Self {
        let mut total = Duration::ZERO;
        let mut min = Duration::MAX;
        let mut max = Duration::ZERO;
        for sample in samples {
            total += *sample;
            min = min.min(*sample);
            max = max.max(*sample);
        }
        Self {
            total,
            min,
            max,
            count: samples.len(),
        }
    }

    fn total_ms(&self) -> f64 {
        self.total.as_secs_f64() * 1000.0
    }

    fn mean_ms(&self) -> f64 {
        self.total_ms() / self.count as f64
    }

    fn min_ms(&self) -> f64 {
        self.min.as_secs_f64() * 1000.0
    }

    fn max_ms(&self) -> f64 {
        self.max.as_secs_f64() * 1000.0
    }
}
