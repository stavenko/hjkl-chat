mod test_utils;

use futures_util::stream::StreamExt;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::types::{S3Api, ToStream};
use minio::s3::ClientBuilder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::fs;
use std::str::FromStr;
use test_utils::{generate_random_bucket_prefix, generate_temp_sqlite_path, generate_unique_email};

#[derive(Deserialize)]
struct TestConfig {
    s3: S3Config,
    smtp: SmtpConfig,
    sqlite: SqliteConfig,
}

#[derive(Deserialize)]
struct S3Config {
    endpoint: String,
    access_key: String,
    secret_key: String,
    bucket: String,
}

#[derive(Deserialize)]
struct SmtpConfig {
    host: String,
    port: u16,
    #[allow(dead_code)]
    username: String,
    #[allow(dead_code)]
    password: String,
}

#[derive(Deserialize)]
struct SqliteConfig {
    path: String,
}

#[derive(Serialize, Deserialize)]
struct MailHogMessage {
    id: u64,
    from: String,
    to: Vec<String>,
    subject: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct MailHogList {
    items: Vec<MailHogItem>,
}

#[derive(Serialize, Deserialize)]
struct MailHogItem {
    id: u64,
    from: String,
    to: Vec<String>,
    subject: String,
}

#[test]
fn test_isolation_helpers_generate_unique_values() {
    let prefix1 = generate_random_bucket_prefix();
    let prefix2 = generate_random_bucket_prefix();
    assert_ne!(prefix1, prefix2, "Bucket prefixes should be unique");
    assert!(prefix1.starts_with("test-"), "Bucket prefix should start with 'test-'");

    let path1 = generate_temp_sqlite_path();
    let path2 = generate_temp_sqlite_path();
    assert_ne!(path1, path2, "SQLite paths should be unique");
    assert!(path1.to_string_lossy().contains(".db"), "Path should end with .db");

    let email1 = generate_unique_email();
    let email2 = generate_unique_email();
    assert_ne!(email1, email2, "Emails should be unique");
    assert!(email1.contains("@example.com"), "Email should be valid");
}

#[test]
fn test_config_loading() {
    let config_path = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/../tests/config.toml", config_path);
    let config_content = fs::read_to_string(&config_path).expect("Failed to read config file");
    let config: TestConfig = toml::from_str(&config_content).expect("Failed to parse config");

    assert!(!config.s3.endpoint.is_empty(), "S3 endpoint must be configured");
    assert!(!config.s3.access_key.is_empty(), "S3 access_key must be configured");
    assert!(!config.s3.secret_key.is_empty(), "S3 secret_key must be configured");
    assert!(!config.s3.bucket.is_empty(), "S3 bucket must be configured");

    assert!(!config.smtp.host.is_empty(), "SMTP host must be configured");
    assert!(config.smtp.port > 0, "SMTP port must be configured");

    assert!(!config.sqlite.path.is_empty(), "SQLite path must be configured");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_minio_s3_integration() {
    let config_path = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/../tests/config.toml", config_path);
    let config_content = fs::read_to_string(&config_path).expect("Failed to read config file");
    let config: TestConfig = toml::from_str(&config_content).expect("Failed to parse config");

    let base_url = BaseUrl::from_str(&config.s3.endpoint)
        .expect("Failed to parse MinIO endpoint");
    let provider = StaticProvider::new(&config.s3.access_key, &config.s3.secret_key, None);

    let client = ClientBuilder::new(base_url)
        .provider(Some(Box::new(provider)))
        .build()
        .expect("Failed to build MinIO client");

    let bucket_prefix = generate_random_bucket_prefix();
    let bucket_name = format!(
        "{}{}",
        bucket_prefix,
        uuid::Uuid::new_v4().to_string()[..8].to_string()
    );

    let test_content = format!("test-content-{}", uuid::Uuid::new_v4());
    let object_key = format!(
        "test-{}.txt",
        uuid::Uuid::new_v4().to_string()[..8].to_string()
    );

    client
        .create_bucket(&bucket_name)
        .send()
        .await
        .expect("Failed to create bucket");

    let content_bytes = minio::s3::segmented_bytes::SegmentedBytes::from(test_content.clone());
    client
        .put_object(&bucket_name, &object_key, content_bytes)
        .send()
        .await
        .expect("Failed to upload object");

    let get_response = client
        .get_object(&bucket_name, &object_key)
        .send()
        .await
        .expect("Failed to get object");
    let downloaded_bytes = get_response.content.to_segmented_bytes().await.expect("Failed to read content");
    let bytes_vec = downloaded_bytes.to_bytes();
    let downloaded_content = String::from_utf8_lossy(&bytes_vec);
    assert_eq!(
        downloaded_content, test_content,
        "Downloaded content should match uploaded content"
    );

    let list_builder = client.list_objects(&bucket_name);
    let mut stream = list_builder.to_stream().await;
    let objects = stream.next().await.expect("Stream ended").expect("Failed to list objects");
    assert!(
        !objects.contents.is_empty(),
        "Bucket should contain at least one object"
    );

    client
        .remove_object(&bucket_name, &object_key)
        .send()
        .await
        .expect("Failed to delete object");

    let list_builder_after = client.list_objects(&bucket_name);
    let mut stream_after = list_builder_after.to_stream().await;
    let objects_after_delete = stream_after.next().await.expect("Stream ended").expect("Failed to list objects after delete");
    assert!(
        objects_after_delete.contents.is_empty(),
        "Bucket should be empty after deleting object"
    );

    client
        .delete_bucket(&bucket_name)
        .send()
        .await
        .expect("Failed to delete bucket");

    let buckets = client
        .list_buckets()
        .send()
        .await
        .expect("Failed to list buckets");
    assert!(
        !buckets.buckets.iter().any(|b| b.name == bucket_name),
        "Bucket should not exist after deletion"
    );
}

#[tokio::test]
async fn test_mailhog_smtp_integration() {
    use lettre::AsyncTransport;

    let config_path = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/../tests/config.toml", config_path);
    let config_content = fs::read_to_string(&config_path).expect("Failed to read config file");
    let config: TestConfig = toml::from_str(&config_content).expect("Failed to parse config");

    let unique_email = generate_unique_email();
    let test_subject = format!(
        "Test Email {}",
        uuid::Uuid::new_v4().to_string()[..8].to_string()
    );
    let test_body = format!("Test body content {}", uuid::Uuid::new_v4());

    let mailer = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous("localhost")
        .port(config.smtp.port)
        .build();

    let message = lettre::Message::builder()
        .from("sender@example.com".parse().unwrap())
        .to(unique_email.parse().unwrap())
        .subject(&test_subject)
        .body(test_body.clone())
        .unwrap();

    mailer
        .send(message)
        .await
        .expect("Failed to send email");

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let http_client = HttpClient::new();
    let mailhog_url = format!("http://{}:8025/api/v2/messages", config.smtp.host);
    let response: MailHogList = http_client
        .get(&mailhog_url)
        .send()
        .await
        .expect("Failed to fetch MailHog messages")
        .json()
        .await
        .expect("Failed to parse MailHog response");

    let found_message = response
        .items
        .iter()
        .find(|item| item.to.iter().any(|to| to == &unique_email));
    assert!(
        found_message.is_some(),
        "Email should be found in MailHog"
    );

    let message_id = found_message.unwrap().id;
    let message_detail_url = format!(
        "http://{}:8025/api/v2/messages/{}",
        config.smtp.host, message_id
    );
    let message_detail: MailHogMessage = http_client
        .get(&message_detail_url)
        .send()
        .await
        .expect("Failed to fetch MailHog message detail")
        .json()
        .await
        .expect("Failed to parse MailHog message detail");

    assert_eq!(
        message_detail.from, "sender@example.com",
        "From address should match"
    );
    assert!(
        message_detail.to.iter().any(|to| to == &unique_email),
        "To address should match"
    );
    assert_eq!(
        message_detail.subject, test_subject,
        "Subject should match"
    );
    assert!(
        message_detail.content.contains(&test_body),
        "Content should match"
    );
}

#[tokio::test]
async fn test_config_missing_required_fields() {
    let invalid_config_content = r#"
[s3]
endpoint = "http://localhost:9000"
"#;

    let result: Result<TestConfig, _> = toml::from_str(invalid_config_content);
    assert!(
        result.is_err(),
        "Config parsing should fail when required fields are missing"
    );
}
