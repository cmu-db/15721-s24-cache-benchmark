use istziio_client::client_api::{StorageClient, TableId};
use istziio_client::storage_client::StorageClientImpl;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn setup_client_1() -> Box<dyn StorageClient> {
    let server_url = std::env::var("SERVER_URL").unwrap_or(String::from("http://127.0.0.1:26379"));
    println!("server url: {}", &server_url);
    let bench_files_path = "./bench_files".to_string();
    let map = create_table_file_map(&bench_files_path).unwrap();
    Box::new(StorageClientImpl::new_for_test(1, map.clone(), &server_url))
}

pub fn setup_client_2() -> Box<dyn StorageClient> {
    let server_url = std::env::var("SERVER_URL").unwrap_or(String::from("http://127.0.0.1:26379"));
    println!("server url: {}", &server_url);
    let bench_files_path = "./bench_files".to_string();
    let map = create_table_file_map(&bench_files_path).unwrap();
    Box::new(StorageClientImpl::new_for_test(1, map.clone(), &server_url))
}

pub fn create_table_file_map(directory: &str) -> Result<HashMap<TableId, String>, std::io::Error> {
    let mut table_file_map: HashMap<TableId, String> = HashMap::new();
    let dir = Path::new(directory);

    // Read the directory entries
    let entries = fs::read_dir(dir)?;

    // Iterate over the entries
    for (id, entry) in entries.enumerate() {
        let entry = entry?;
        if entry.path().is_file() {
            // If the entry is a file, add it to the map with an incremental ID
            let filename = entry.file_name().into_string().unwrap();
            table_file_map.insert(id as TableId, filename);
        }
    }

    Ok(table_file_map)
}
