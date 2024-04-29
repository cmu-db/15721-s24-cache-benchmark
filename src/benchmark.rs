use csv;
use istziio_client::client_api::{StorageClient, StorageRequest};
use std::collections::VecDeque;
use std::error::Error;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Instant;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;

// This scans the bench_files dir to figure out which test files are present,
// then builds a map of TableId -> filename to init storage client(only when catalog is not available)
// and also generates workload based on table ids. Finally it runs the workload

pub struct TraceEntry {
    pub timestamp: u64,
    pub request: StorageRequest,
}

pub enum ClientType {
    Client1(),
    Client2(),
}

pub fn parse_trace(trace_path: PathBuf) -> Result<VecDeque<TraceEntry>, Box<dyn Error>> {
    let mut trace: VecDeque<TraceEntry> = VecDeque::new();
    let mut rdr = csv::Reader::from_path(trace_path)?;
    for result in rdr.records() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here.
        let record = result?;
        trace.push_back(TraceEntry {
            timestamp: record.get(0).unwrap().parse().unwrap(),
            request: StorageRequest::Table(record.get(1).unwrap().parse().unwrap()),
        });
    }
    Ok(trace)
}
pub async fn run_trace(
    mut trace: VecDeque<TraceEntry>,
    client_builder: &dyn Fn() -> Box<dyn StorageClient>,
) {
    let start_time = SystemTime::now();
    let request_num = trace.len();
    let (tx, mut rx) = mpsc::channel(32);
    while !trace.is_empty() {
        let next_entry = trace.pop_front().unwrap();
        if let Some(diff) =
            Duration::from_millis(next_entry.timestamp).checked_sub(start_time.elapsed().unwrap())
        {
            sleep(diff);
        }
        println!("next trace: {}", next_entry.timestamp);
        let tx = tx.clone();
        let client = client_builder();
        tokio::spawn(async move {
            let table_id = match next_entry.request {
                StorageRequest::Table(id) => id,
                _ => panic!("Invalid request type"),
            };
            println!("start thread reading {}", table_id);
            let client_start = Instant::now();
            let req = next_entry.request;

            let res = client.request_data(req.clone()).await;
            if let Err(e) = res {
                println!("Error: {}", e);
            }
            let client_duration = client_start.elapsed();
            tx.send(client_duration).await.unwrap();
        });
    }

    // Collect and print client latencies
    let mut duration_sum = Duration::new(0, 0);
    for _ in 0..request_num {
        let client_duration = rx.recv().await.unwrap();
        println!("Client latency: {:?}", client_duration);
        duration_sum += client_duration;
    }

    let avg_duration = duration_sum.div_f32(request_num as f32);
    println!("Average duration: {:?}", avg_duration);
}
