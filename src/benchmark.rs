use istziio_client::client_api::{StorageClient, StorageRequest};
use prettytable::{Cell, Row, Table};
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
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

pub fn parse_trace(trace_path: PathBuf) -> Result<Vec<TraceEntry>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(trace_path)?;
    let mut traces = Vec::new();
    for result in rdr.records() {
        let record = result?;
        traces.push(TraceEntry {
            timestamp: record.get(0).unwrap().parse().unwrap(),
            request: StorageRequest::Table(record.get(1).unwrap().parse().unwrap()),
        });
    }
    Ok(traces)
}

pub async fn run_trace(
    traces: Vec<TraceEntry>,
    client_builder: &dyn Fn() -> Box<dyn StorageClient>,
) {
    let start_time = SystemTime::now();
    let request_num = traces.len();
    let (tx, mut rx) = mpsc::channel(32);
    let table = Arc::new(Mutex::new(Table::new()));
    table.lock().unwrap().add_row(Row::new(vec![
        Cell::new("Trace ID"),
        Cell::new("File ID"),
        Cell::new("Num Rows"),
        Cell::new("Arrival Time"),
        Cell::new("Wait Time"),
    ]));
    for (i, trace) in traces.into_iter().enumerate() {
        let table = Arc::clone(&table);
        if let Some(diff) =
            Duration::from_millis(trace.timestamp).checked_sub(start_time.elapsed().unwrap())
        {
            sleep(diff);
        }
        let tx = tx.clone();
        let client = client_builder();
        tokio::spawn(async move {
            let table_id = match trace.request {
                StorageRequest::Table(id) => id,
                _ => panic!("Invalid request type"),
            };
            // println!(
            //     "Trace {} sends request for table {} at timestamp {}",
            //     i, table_id, trace.timestamp
            // );
            let client_start = Instant::now();

            let res = client.request_data(trace.request).await;
            if res.is_err() {
                println!("Error: {}", res.as_ref().err().unwrap());
            }
            let mut rx = res.unwrap();
            let mut total_num_rows = 0;
            while let Some(rb) = rx.recv().await {
                total_num_rows += rb.num_rows();
            }
            let client_duration = client_start.elapsed();
            // println!(
            //     "Trace {} gets {} rows from the client, latency is {:?}",
            //     i, total_num_rows, client_duration
            // );
            table.lock().unwrap().add_row(Row::new(vec![
                Cell::new(&i.to_string()),
                Cell::new(&table_id.to_string()),
                Cell::new(&total_num_rows.to_string()),
                Cell::new(&trace.timestamp.to_string()),
                Cell::new(&client_duration.as_millis().to_string()),
            ]));
            tx.send(client_duration).await.unwrap();
        });
    }

    // Collect and print client latencies
    let mut duration_sum = Duration::new(0, 0);
    for _ in 0..request_num {
        let client_duration = rx.recv().await.unwrap();
        duration_sum += client_duration;
    }

    let avg_duration = duration_sum.div_f32(request_num as f32);
    table.lock().unwrap().printstd();
    println!("Average duration: {:?}", avg_duration);
}
