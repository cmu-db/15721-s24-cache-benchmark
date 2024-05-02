use client_benchmark::{
    benchmark::{parse_trace, run_trace},
    utils,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // benchmark_sync().await;
    // benchmark_parallel().await;
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();

    // let trace = parse_trace(PathBuf::from("traces/trace_1m.csv")).unwrap();
    // run_trace(trace, &utils::setup_client_1).await;
    let trace = parse_trace(PathBuf::from("traces/trace_100m.csv")).unwrap();
    run_trace(trace, &utils::setup_client_1).await;
    // let trace = parse_trace(PathBuf::from("traces/trace_parallel.csv")).unwrap();
    // run_trace(trace, &utils::setup_client_1).await;
    // let trace = parse_trace(PathBuf::from("traces/trace_serial.csv")).unwrap();
    // run_trace(trace, &utils::setup_client_1).await;
}
