use client_benchmark::{
    benchmark::{parse_trace, run_trace},
    utils,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // benchmark_sync().await;
    // benchmark_parallel().await;
    let trace = parse_trace(PathBuf::from("traces/trace_1m_parallel.csv")).unwrap();
    run_trace(trace, &utils::setup_client_1).await;
}
