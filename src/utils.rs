use istziio_client::client_api::{StorageClient, TableId};
use istziio_client::storage_client::StorageClientImpl;
use std::collections::HashMap;

pub fn setup_client_1() -> Box<dyn StorageClient> {
    let server_url = std::env::var("SERVER_URL").unwrap_or(String::from("http://127.0.0.1:26379"));
    println!("server url: {}", &server_url);
    let map = create_table_file_map().unwrap();
    Box::new(StorageClientImpl::new_for_test(1, map.clone(), &server_url))
}

pub fn setup_client_2() -> Box<dyn StorageClient> {
    let server_url = std::env::var("SERVER_URL").unwrap_or(String::from("http://127.0.0.1:26379"));
    println!("server url: {}", &server_url);
    let map = create_table_file_map().unwrap();
    Box::new(StorageClientImpl::new_for_test(1, map.clone(), &server_url))
}

pub fn create_table_file_map() -> Result<HashMap<TableId, String>, std::io::Error> {
    let mut table_file_map: HashMap<TableId, String> = HashMap::new();

    // Iterate over the entries
    for id in 0..5 {
        table_file_map.insert(id as TableId, format!("1000000row_10col_{}.parquet", id));
    }

    Ok(table_file_map)
}
