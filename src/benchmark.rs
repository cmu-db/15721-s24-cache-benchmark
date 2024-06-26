use istziio_client::client_api::{DataRequest, StorageClient, StorageRequest};
use prettytable::{Cell, Row, Table};
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

pub fn parse_trace(trace_path: PathBuf) -> Result<Vec<TraceEntry>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(trace_path)?;
    let mut traces = Vec::new();
    let mut req_id: usize = 0;
    for result in rdr.records() {
        let record = result?;
        traces.push(TraceEntry {
            timestamp: record.get(0).unwrap().parse().unwrap(),
            request: StorageRequest::new(
                req_id,
                DataRequest::Table(record.get(1).unwrap().parse().unwrap()),
            ),
        });
        req_id += 1;
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
    for (i, trace) in traces.into_iter().enumerate() {
        if let Some(diff) =
            Duration::from_millis(trace.timestamp).checked_sub(start_time.elapsed().unwrap())
        {
            sleep(diff);
        }
        let tx = tx.clone();
        let client = client_builder();
        tokio::spawn(async move {
            let table_id = match trace.request.data_request() {
                DataRequest::Table(id) => id.clone(),
                _ => panic!("Invalid request type"),
            };
            let client_start = Instant::now();

            let res = client.request_data(trace.request.clone()).await;
            if res.is_err() {
                println!("Error: {}", res.as_ref().err().unwrap());
            }
            let mut rx = res.unwrap();
            let mut total_num_rows = 0;
            while let Some(rb) = rx.recv().await {
                total_num_rows += rb.num_rows();
            }
            let client_duration = client_start.elapsed();
            tx.send((
                i,
                table_id,
                total_num_rows,
                trace.timestamp,
                client_duration,
            ))
            .await
            .unwrap();
        });
    }

    // Collect and print client latencies
    let mut duration_sum = Duration::new(0, 0);
    let mut rows = Vec::new();
    for _ in 0..request_num {
        let tuple = rx.recv().await.unwrap();
        rows.push(tuple);
        duration_sum += tuple.4;
    }

    // Sort rows based on the first element of the tuple
    rows.sort_by(|a, b| a.0.cmp(&b.0));

    // Construct a table to print the results
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Trace ID"),
        Cell::new("File ID"),
        Cell::new("Num Rows"),
        Cell::new("Arrival Time"),
        Cell::new("Wait Time"),
    ]));

    for row in rows {
        table.add_row(Row::new(vec![
            Cell::new(&row.0.to_string()),
            Cell::new(&row.1.to_string()),
            Cell::new(&row.2.to_string()),
            Cell::new(&row.3.to_string()),
            Cell::new(&row.4.as_millis().to_string()),
        ]));
    }

    let avg_duration = duration_sum.div_f32(request_num as f32);
    table.printstd();
    println!("Average duration: {:?}", avg_duration);
}
