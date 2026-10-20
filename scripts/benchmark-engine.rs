//! Standalone benchmark driver; compile against the release library (see report).
use fluidise_clint::{check_message, prepare_policy};
use std::{hint::black_box, time::Instant};

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    assert_eq!(args.len(), 2, "expected message file and iteration count");
    let text = std::fs::read_to_string(&args[0]).unwrap();
    let iterations: u32 = args[1].to_str().unwrap().parse().unwrap();
    assert!(iterations > 0);
    let policy = prepare_policy(None).unwrap();
    for _ in 0..3 {
        black_box(check_message(black_box(&text), black_box(&policy)));
    }
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(check_message(black_box(&text), black_box(&policy)));
    }
    let elapsed = start.elapsed().as_nanos();
    println!("{elapsed}");
    // Full result comparison is outside the measured region.
    println!("{:?}", check_message(&text, &policy));
}
