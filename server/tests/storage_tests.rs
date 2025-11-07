#![cfg(feature = "persistent")]

use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{Client, config::Builder};
use bytes::Bytes;
use file_server_server::repositories::{FileStorage, S3FileStorage};
use uuid::Uuid;

async fn setup_s3_storage() -> S3FileStorage {
    let s3_region = std::env::var("AWS_REGION").unwrap();
    let s3_endpoint = std::env::var("AWS_ENDPOINT_URL").unwrap();
    let s3_bucket = std::env::var("S3__BUCKET").unwrap();

    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(s3_region))
        .endpoint_url(s3_endpoint)
        .load()
        .await;
    let conf = Builder::from(&aws_config).force_path_style(true).build();

    let aws_client = Client::from_conf(conf);

    let check_bucket_exists = aws_client.head_bucket().bucket(&s3_bucket).send().await;
    if check_bucket_exists.is_err() {
        aws_client
            .create_bucket()
            .bucket(&s3_bucket)
            .send()
            .await
            .expect("Failed to create S3 bucket");
    }

    S3FileStorage::from_client(aws_client, s3_bucket)
}

#[tokio::test]
async fn test_full_operations_for_s3_file_storage() {
    dotenv::dotenv().ok();

    let storage = setup_s3_storage().await;

    let id = Uuid::new_v4();

    let get_result = storage.get_file_content(id, "invalid_file.txt").await;
    assert!(matches!(get_result, Ok(None)));

    let file_1 = "file1.txt";
    let ccontents_file_1 = Bytes::from_static(b"contents_file_1");
    let first_insert_result = storage
        .insert_file_content(id, file_1, &ccontents_file_1)
        .await;
    assert!(first_insert_result.is_ok());

    let file_2 = "file2.txt";
    let ccontents_file_2 = Bytes::from_static(b"contents_file_2");
    let second_insert_result = storage
        .insert_file_content(id, file_2, &ccontents_file_2)
        .await;
    assert!(second_insert_result.is_ok());

    let file_1_get_result = storage.get_file_content(id, file_1).await;
    assert!(matches!(file_1_get_result, Ok(Some(contents)) if contents == ccontents_file_1));

    let file_2_get_result = storage.get_file_content(id, file_2).await;
    assert!(matches!(file_2_get_result, Ok(Some(contents)) if contents == ccontents_file_2));
}
