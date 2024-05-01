use client_benchmark::{
    benchmark::{parse_trace, run_trace},
    utils,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    // benchmark_sync().await;
    // benchmark_parallel().await;
    let trace = parse_trace(PathBuf::from("traces/trace_1m.csv")).unwrap();
    run_trace(trace, &utils::setup_client_1).await;
    let trace = parse_trace(PathBuf::from("traces/trace_100m.csv")).unwrap();
    run_trace(trace, &utils::setup_client_1).await;
    let trace = parse_trace(PathBuf::from("traces/trace_parallel.csv")).unwrap();
    run_trace(trace, &utils::setup_client_1).await;
}
