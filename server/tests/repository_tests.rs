#![cfg(feature = "persistent")]

use clickhouse::Client;
use file_server_library::models::Hash32;
use file_server_server::repositories::{
    ClickhouseConfig, ClickhouseFileRepository, FileMerkleTreeRow, FileRepository,
};
use std::collections::HashMap;
use uuid::Uuid;

const TABLE_NAME: &str = "files";

async fn clear_files_table(config: &ClickhouseConfig) {
    let client = Client::default()
        .with_url(config.database_url.to_owned())
        .with_user(config.username.to_owned())
        .with_password(config.password.to_owned())
        .with_database(config.database_name.to_owned());

    let _ = client
        .with_option("mutations_sync", "1")
        .query(&format!("TRUNCATE TABLE {TABLE_NAME}"))
        .execute()
        .await;
}

#[tokio::test]
async fn test_full_operations_for_clickhouse_repository() {
    dotenv::dotenv().ok();

    let config = ClickhouseConfig::load_from_env().unwrap();

    clear_files_table(&config).await;

    let repo = ClickhouseFileRepository::new(config);

    let row = FileMerkleTreeRow {
        id: Uuid::new_v4(),
        order: vec!["file1.txt".to_string(), "file2.txt".to_string()],
        files: HashMap::from([
            (
                "file1.txt".to_string(),
                Hash32::hash("contents_file_1".as_bytes()),
            ),
            (
                "file2.txt".to_string(),
                Hash32::hash("contents_file_2".as_bytes()),
            ),
        ]),
        leaf_hashes: vec![
            Hash32::hash("contents_file_1".as_bytes()),
            Hash32::hash("contents_file_2".as_bytes()),
        ],
        root: None,
    };

    let insert_result = repo.insert(row.clone()).await;
    assert!(insert_result.is_ok());

    let get_result = repo.get(row.id).await;
    assert!(get_result.is_ok());

    let fetched_row = get_result.unwrap().unwrap();
    assert_eq!(fetched_row, row);

    let updated_row = FileMerkleTreeRow {
        id: row.id,
        order: vec![
            "file1.txt".to_string(),
            "file2.txt".to_string(),
            "file3.txt".to_string(),
        ],
        files: HashMap::from([
            (
                "file1.txt".to_string(),
                Hash32::hash("contents_file_1".as_bytes()),
            ),
            (
                "file2.txt".to_string(),
                Hash32::hash("contents_file_2".as_bytes()),
            ),
            (
                "file3.txt".to_string(),
                Hash32::hash("contents_file_3".as_bytes()),
            ),
        ]),
        leaf_hashes: vec![
            Hash32::hash("contents_file_1".as_bytes()),
            Hash32::hash("contents_file_2".as_bytes()),
            Hash32::hash("contents_file_3".as_bytes()),
        ],
        root: None,
    };
    let update_result = repo.update(updated_row.clone()).await;
    assert!(update_result.is_ok());

    let get_updated_result = repo.get(row.id).await;
    assert!(get_updated_result.is_ok());

    let fetched_updated_row = get_updated_result.unwrap().unwrap();
    assert_eq!(fetched_updated_row, updated_row);
}
