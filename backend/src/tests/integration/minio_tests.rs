use aws_sdk_s3::config::Region;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use std::time::Duration;

use crate::tests::utils::random_bucket_prefix;

const MINIO_ENDPOINT: &str = "http://localhost:9000";
const MINIO_ACCESS_KEY: &str = "minioadmin";
const MINIO_SECRET_KEY: &str = "minioadmin";

async fn create_s3_client() -> S3Client {
    let region = Region::new("us-east-1");

    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region)
        .endpoint_url(MINIO_ENDPOINT)
        .credentials_provider(
            aws_sdk_s3::config::Credentials::new(
                MINIO_ACCESS_KEY,
                MINIO_SECRET_KEY,
                None,
                None,
                "test",
            ),
        )
        .load()
        .await;

    S3Client::new(&config)
}

async fn wait_for_minio_health_check(max_retries: u32) -> Result<(), String> {
    for _i in 0..max_retries {
        match reqwest::get(format!("{}/minio/health/live", MINIO_ENDPOINT))
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(());
                }
            }
            Err(_) => {}
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Err(format!("MinIO health check failed after {} retries", max_retries))
}

#[actix_rt::test]
#[ignore = "Requires MinIO service running"]
async fn test_minio_health_check() {
    let result = wait_for_minio_health_check(10).await;
    assert!(
        result.is_ok(),
        "MinIO should be available. Error: {:?}",
        result.err()
    );
}

#[actix_rt::test]
#[ignore = "Requires MinIO service running"]
async fn test_minio_create_bucket() {
    let bucket_prefix = random_bucket_prefix();
    let bucket_name = format!("{}{}", bucket_prefix, uuid::Uuid::new_v4().simple());

    let result = create_s3_client()
        .await
        .create_bucket()
        .bucket(&bucket_name)
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Should be able to create bucket. Error: {:?}",
        result.err()
    );

    let client = create_s3_client().await;
    client
        .delete_bucket()
        .bucket(&bucket_name)
        .send()
        .await
        .ok();
}

#[actix_rt::test]
#[ignore = "Requires MinIO service running"]
async fn test_minio_upload_download_object() {
    let bucket_prefix = random_bucket_prefix();
    let bucket_name = format!("{}{}", bucket_prefix, uuid::Uuid::new_v4().simple());
    let object_key = "test-object.txt";
    let object_content = b"Hello, MinIO!";

    let client = create_s3_client().await;

    client
        .create_bucket()
        .bucket(&bucket_name)
        .send()
        .await
        .expect("Should create bucket");

    client
        .put_object()
        .bucket(&bucket_name)
        .key(object_key)
        .body(ByteStream::from(object_content.to_vec()))
        .send()
        .await
        .expect("Should upload object");

    let get_result = client
        .get_object()
        .bucket(&bucket_name)
        .key(object_key)
        .send()
        .await;

    assert!(
        get_result.is_ok(),
        "Should be able to download object. Error: {:?}",
        get_result.err()
    );

    let response = get_result.unwrap();
    let body = response.body.collect().await.expect("Should collect body");
    let bytes = body.into_bytes();

    assert_eq!(
        &bytes[..],
        object_content,
        "Downloaded content should match uploaded content"
    );

    client.delete_object().bucket(&bucket_name).key(object_key).send().await.ok();
    client.delete_bucket().bucket(&bucket_name).send().await.ok();
}

#[actix_rt::test]
#[ignore = "Requires MinIO service running"]
async fn test_minio_delete_bucket() {
    let bucket_prefix = random_bucket_prefix();
    let bucket_name = format!("{}{}", bucket_prefix, uuid::Uuid::new_v4().simple());

    let client = create_s3_client().await;

    client
        .create_bucket()
        .bucket(&bucket_name)
        .send()
        .await
        .expect("Should create bucket");

    let delete_result = client.delete_bucket().bucket(&bucket_name).send().await;

    assert!(
        delete_result.is_ok(),
        "Should be able to delete bucket. Error: {:?}",
        delete_result.err()
    );
}

#[actix_rt::test]
#[ignore = "Requires MinIO service running"]
async fn test_minio_bucket_cleanup() {
    let bucket_prefix = random_bucket_prefix();
    let bucket_name = format!("{}{}", bucket_prefix, uuid::Uuid::new_v4().simple());

    let client = create_s3_client().await;

    client
        .create_bucket()
        .bucket(&bucket_name)
        .send()
        .await
        .expect("Should create bucket");

    client
        .put_object()
        .bucket(&bucket_name)
        .key("test-key")
        .body(ByteStream::from("test-content".as_bytes().to_vec()))
        .send()
        .await
        .expect("Should upload object");

    client.delete_object().bucket(&bucket_name).key("test-key").send().await.ok();
    client.delete_bucket().bucket(&bucket_name).send().await.ok();

    let head_result = client.head_bucket().bucket(&bucket_name).send().await;

    assert!(
        head_result.is_err(),
        "Bucket should be deleted. Error: {:?}",
        head_result.err()
    );
}