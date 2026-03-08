use lettre::Tokio1Executor;
use lettre::message::{header::ContentType, Mailbox, Message, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::AsyncSmtpTransport;
use lettre::AsyncTransport;
use std::time::Duration;

use crate::tests::utils::unique_email;

const MAILHOG_SMTP_HOST: &str = "localhost";
const MAILHOG_API_HOST: &str = "http://localhost:8025";

async fn wait_for_mailhog_health_check(max_retries: u32) -> Result<(), String> {
    for _i in 0..max_retries {
        match reqwest::get(format!("{}/api/v2/messages", MAILHOG_API_HOST)).await {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(());
                }
            }
            Err(_) => {}
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Err(format!("MailHog health check failed after {} retries", max_retries))
}

pub async fn send_email(to_email: &str, subject: &str, body: &str) -> Result<(), lettre::error::Error> {
    let creds = Credentials::new("test".to_string(), "test".to_string());

    let mail = Message::builder()
        .from("test@example.com".parse::<Mailbox>().unwrap())
        .to(to_email.parse::<Mailbox>().unwrap())
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(body.to_string()),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::parse("text/html").unwrap())
                        .body(format!("<p>{}</p>", body)),
                ),
        )
        .unwrap();

    let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(MAILHOG_SMTP_HOST)
        .port(1025)
        .credentials(creds)
        .build();

    transport.send(mail).await.map_err(|e| lettre::error::Error::from(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    )))?;
    Ok(())
}

pub async fn get_mailhog_messages(email: &str) -> Result<Vec<serde_json::Value>, reqwest::Error> {
    let url = format!("{}/api/v2/messages", MAILHOG_API_HOST);
    let response = reqwest::get(&url).await?;
    let body: serde_json::Value = response.json().await?;

    let items = body.get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let filtered: Vec<serde_json::Value> = items
        .into_iter()
        .filter(|msg| {
            msg.pointer("/Content/Headers/To")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().any(|t| {
                    t.as_str().map(|s| s.contains(email)).unwrap_or(false)
                }))
                .unwrap_or(false)
        })
        .collect();

    Ok(filtered)
}

#[actix_rt::test]
async fn test_mailhog_health_check() {
    let result = wait_for_mailhog_health_check(10).await;
    assert!(
        result.is_ok(),
        "MailHog should be available. Error: {:?}",
        result.err()
    );
}

#[actix_rt::test]
async fn test_mailhog_send_email() {
    let test_email = unique_email();
    let subject = "Test Email";
    let body = "This is a test email body";

    let send_result = send_email(&test_email, subject, body).await;

    assert!(
        send_result.is_ok(),
        "Should be able to send email. Error: {:?}",
        send_result.err()
    );

    tokio::time::sleep(Duration::from_millis(500)).await;
}

#[actix_rt::test]
async fn test_mailhog_retrieve_emails() {
    let test_email = unique_email();
    let subject = "Retrieve Test";
    let body = "Body for retrieve test";

    send_email(&test_email, subject, body).await.ok();
    tokio::time::sleep(Duration::from_millis(500)).await;

    let messages = get_mailhog_messages(&test_email).await;

    assert!(messages.is_ok(), "Should be able to retrieve messages");
    assert!(
        messages.unwrap().len() >= 1,
        "Should have at least one message for the test email"
    );
}

#[actix_rt::test]
async fn test_mailhog_verify_email_content() {
    let test_email = unique_email();
    let subject = "Content Verification";
    let body = "Verification body content";

    send_email(&test_email, subject, body).await.ok();
    tokio::time::sleep(Duration::from_millis(500)).await;

    let messages = get_mailhog_messages(&test_email).await.unwrap();
    assert!(!messages.is_empty(), "Should have messages");

    let message = messages.first().unwrap();
    let from = message.pointer("/Content/Headers/From/0").and_then(|v| v.as_str()).unwrap();
    let to = message.pointer("/Content/Headers/To/0").and_then(|v| v.as_str()).unwrap();
    let msg_subject = message.pointer("/Content/Headers/Subject/0").and_then(|v| v.as_str()).unwrap();

    assert!(
        from.contains("test@example.com"),
        "Sender should be test@example.com, got: {}",
        from
    );
    assert!(
        to.contains(&test_email),
        "Recipient should contain test email, got: {}",
        to
    );
    assert_eq!(
        msg_subject, subject,
        "Subject should match, expected: {}, got: {}",
        subject, msg_subject
    );
}

#[actix_rt::test]
async fn test_mailhog_multiple_emails() {
    let test_email = unique_email();

    send_email(&test_email, "Email 1", "First email").await.ok();
    send_email(&test_email, "Email 2", "Second email").await.ok();
    send_email(&test_email, "Email 3", "Third email").await.ok();

    tokio::time::sleep(Duration::from_millis(500)).await;

    let messages = get_mailhog_messages(&test_email).await.unwrap();

    assert!(
        messages.len() >= 3,
        "Should have at least 3 messages, got: {}",
        messages.len()
    );
}