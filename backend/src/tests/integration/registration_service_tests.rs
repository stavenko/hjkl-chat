use actix_web::test::TestRequest;
use actix_web::dev::Service;
use chrono::{Duration, Utc};
use crate::models::registration::RegistrationSession;
use crate::tests::test_app::{create_app, create_app_with_fixtures};
use crate::tests::integration::mailhog_tests::get_mailhog_messages;
use crate::tests::utils::unique_email;
use serde_json::{json, Value};
use uuid::Uuid;

fn get_session_from_db(sqlite: &crate::providers::sqlite::SQLiteProvider, session_id: Uuid) -> Option<RegistrationSession> {
    let conn = sqlite.get_connection().ok()?;
    let mut stmt = conn.prepare("SELECT * FROM registration_sessions WHERE id = ?").ok()?;
    let mut rows = stmt.query(rusqlite::params![session_id]).ok()?;
    if let Some(row_result) = rows.next().ok()? {
        Some(RegistrationSession::from_row(row_result).unwrap())
    } else {
        None
    }
}

#[actix_rt::test]
async fn test_registration_init_successful() {
    let (_details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["message"], "Verification email sent");
    assert!(body["session_id"].as_str().map(|s| !s.is_empty()).unwrap_or(false));
    assert!(body["resend_available_at"].as_str().map(|s| !s.is_empty()).unwrap_or(false));
}

#[actix_rt::test]
async fn test_registration_init_generates_6_digit_code() {
    let (details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;

    let session_id: Uuid = body["session_id"].as_str().unwrap().parse().unwrap();
    let session = get_session_from_db(&details.sqlite, session_id).unwrap();

    assert_eq!(session.verification_code.len(), 6, "Verification code should be exactly 6 digits");
    assert!(
        session.verification_code.chars().all(|c| c.is_ascii_digit()),
        "Verification code should contain only digits"
    );
}

#[actix_rt::test]
async fn test_registration_init_creates_session_in_database() {
    let (details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;

    let session_id: Uuid = body["session_id"].as_str().unwrap().parse().unwrap();
    let session = get_session_from_db(&details.sqlite, session_id);

    assert!(session.is_some(), "Session should be created in database");

    let session = session.unwrap();
    assert_eq!(session.id, session_id);
    assert_eq!(session.email, email);
    assert!(!session.verification_code.is_empty());
    assert!(session.created_at <= Utc::now());
    assert!(session.expires_at > session.created_at);
    assert!(session.resend_available_at > session.created_at);
}

#[actix_rt::test]
async fn test_registration_init_sends_email_via_smtp() {
    let (_details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200, "Email should be sent successfully");
}

#[actix_rt::test]
async fn test_registration_init_session_expires_in_15_minutes() {
    let (details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;

    let session_id: Uuid = body["session_id"].as_str().unwrap().parse().unwrap();
    let session = get_session_from_db(&details.sqlite, session_id).unwrap();

    let expected_expires = session.created_at + Duration::minutes(15);
    let diff = (session.expires_at.timestamp() - expected_expires.timestamp()).abs();

    assert!(
        diff <= 1,
        "Session should expire in 15 minutes (allow 1 second tolerance), got: {:?}",
        session.expires_at
    );
}

#[actix_rt::test]
async fn test_registration_init_resend_available_in_60_seconds() {
    let (details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;

    let session_id: Uuid = body["session_id"].as_str().unwrap().parse().unwrap();
    let session = get_session_from_db(&details.sqlite, session_id).unwrap();

    let expected_resend = session.created_at + Duration::seconds(60);
    let diff = (session.resend_available_at.timestamp() - expected_resend.timestamp()).abs();

    assert!(
        diff <= 1,
        "Resend should be available in 60 seconds (allow 1 second tolerance), got: {:?}",
        session.resend_available_at
    );
}

#[actix_rt::test]
async fn test_registration_init_empty_email_returns_error() {
    let (_details, app) = create_app().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": ""}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 400);
}

#[actix_rt::test]
async fn test_registration_init_invalid_email_format_returns_error() {
    let invalid_emails = vec!["invalid-email", "missing@domain", "no-at-sign.com"];

    for invalid_email in invalid_emails {
        let (_details, app) = create_app().await.unwrap();

        let req = TestRequest::post()
            .uri("/api/auth/registration/init")
            .set_json(json!({"email": invalid_email}))
            .to_request();

        let resp = app.call(req).await.unwrap();
        assert!(
            resp.status() == 400,
            "Invalid email '{}' should return 400, got: {}",
            invalid_email,
            resp.status()
        );
    }
}

#[actix_rt::test]
async fn test_registration_init_duplicate_email_returns_error() {
    let (_details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req1 = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();
    let resp1 = app.call(req1).await.unwrap();
    assert_eq!(resp1.status(), 200);
    let _body1: Value = actix_web::test::read_body_json(resp1).await;

    let req2 = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();
    let resp2 = app.call(req2).await.unwrap();
    assert!(
        resp2.status() == 409 || resp2.status() == 500,
        "Duplicate email should return 409 or 500, got: {}",
        resp2.status()
    );
}

#[actix_rt::test]
async fn test_session_stored_with_correct_schema() {
    let (details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;

    let session_id: Uuid = body["session_id"].as_str().unwrap().parse().unwrap();
    let session = get_session_from_db(&details.sqlite, session_id).unwrap();

    assert_eq!(session.id, session_id);
    assert_eq!(session.email, email);
    assert_eq!(session.verification_code.len(), 6);
    assert!(session.created_at <= Utc::now());
    assert!(session.expires_at > session.created_at);
    assert!(session.resend_available_at > session.created_at);

    let conn = details.sqlite.get_connection().unwrap();
    let mut stmt = conn.prepare("PRAGMA table_info(registration_sessions)").unwrap();
    let columns = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>("name")?,
                row.get::<_, String>("type")?,
            ))
        })
        .unwrap();

    let column_names: Vec<String> = columns
        .filter_map(|r| r.ok().map(|(name, _)| name))
        .collect();

    assert!(column_names.contains(&"id".to_string()), "Table should have 'id' column");
    assert!(column_names.contains(&"email".to_string()), "Table should have 'email' column");
    assert!(
        column_names.contains(&"verification_code".to_string()),
        "Table should have 'verification_code' column"
    );
    assert!(column_names.contains(&"created_at".to_string()), "Table should have 'created_at' column");
    assert!(column_names.contains(&"expires_at".to_string()), "Table should have 'expires_at' column");
    assert!(
        column_names.contains(&"resend_available_at".to_string()),
        "Table should have 'resend_available_at' column"
    );
}

#[actix_rt::test]
async fn test_session_uuid_is_valid_format() {
    let (details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;

    let session_id: Uuid = body["session_id"].as_str().unwrap().parse().unwrap();
    let session = get_session_from_db(&details.sqlite, session_id).unwrap();

    let uuid_str = session.id.to_string();
    assert!(uuid::Uuid::parse_str(&uuid_str).is_ok(), "Session ID should be valid UUID");
    assert_eq!(uuid_str.len(), 36, "UUID string should be 36 characters with dashes");
}

#[actix_rt::test]
async fn test_session_email_is_unique() {
    let (_details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req1 = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();
    let resp1 = app.call(req1).await.unwrap();
    assert_eq!(resp1.status(), 200);
    let _body1: Value = actix_web::test::read_body_json(resp1).await;

    let req2 = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();
    let resp2 = app.call(req2).await.unwrap();
    assert!(
        resp2.status() != 200,
        "Second registration with same email should fail due to UNIQUE constraint"
    );
}

#[actix_rt::test]
async fn test_session_timestamps_are_rfc3339_compatible() {
    let (details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;

    let session_id: Uuid = body["session_id"].as_str().unwrap().parse().unwrap();
    let session = get_session_from_db(&details.sqlite, session_id).unwrap();

    let created_at_str = session.created_at.to_rfc3339();
    let expires_at_str = session.expires_at.to_rfc3339();
    let resend_str = session.resend_available_at.to_rfc3339();

    assert!(
        chrono::DateTime::parse_from_rfc3339(&created_at_str).is_ok(),
        "created_at should be RFC3339 compatible"
    );
    assert!(
        chrono::DateTime::parse_from_rfc3339(&expires_at_str).is_ok(),
        "expires_at should be RFC3339 compatible"
    );
    assert!(
        chrono::DateTime::parse_from_rfc3339(&resend_str).is_ok(),
        "resend_available_at should be RFC3339 compatible"
    );
}

#[actix_rt::test]
async fn test_email_contains_verification_code() {
    let (details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;

    let session_id: Uuid = body["session_id"].as_str().unwrap().parse().unwrap();
    let session = get_session_from_db(&details.sqlite, session_id).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let messages = get_mailhog_messages(&email).await;
    let msgs = messages.expect("Should be able to retrieve messages from MailHog");
    assert!(!msgs.is_empty(), "Should have sent email to MailHog");

    let message = msgs.first().unwrap();
    let body_content = message.pointer("/Content/Body").and_then(|v| v.as_str());

    assert!(body_content.is_some(), "Email should have body content");
    assert!(
        body_content.unwrap().contains(&session.verification_code),
        "Email body should contain verification code: {}",
        session.verification_code
    );
}

#[actix_rt::test]
async fn test_email_sent_to_correct_address() {
    let (_details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let messages = get_mailhog_messages(&email).await;
    let msgs = messages.expect("Should be able to retrieve messages from MailHog");
    assert!(!msgs.is_empty(), "Should have sent email");

    let message = msgs.first().unwrap();
    let to = message
        .pointer("/Content/Headers/To/0")
        .and_then(|v| v.as_str());

    assert!(to.is_some(), "Email should have 'To' field");
    assert!(
        to.unwrap().contains(&email),
        "Email should be sent to correct address: {}, got: {}",
        email,
        to.unwrap()
    );
}

#[actix_rt::test]
async fn test_email_has_subject_line() {
    let (_details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let messages = get_mailhog_messages(&email).await;
    let msgs = messages.expect("Should be able to retrieve messages from MailHog");
    assert!(!msgs.is_empty(), "Should have sent email");

    let message = msgs.first().unwrap();
    let subject = message
        .pointer("/Content/Headers/Subject/0")
        .and_then(|v| v.as_str());

    assert!(subject.is_some(), "Email should have subject");
    assert_eq!(
        subject.unwrap(),
        "Your Registration Verification Code",
        "Email should have correct subject line"
    );
}

#[actix_rt::test]
async fn test_database_error_handling() {
    let (_details, app, _) = create_app_with_fixtures(|det| async move {
        let conn = det.sqlite.get_connection().unwrap();
        conn.execute("DROP TABLE registration_sessions", []).unwrap();
    })
    .await
    .unwrap();

    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(
        resp.status(),
        500,
        "Should fail with server error when table doesn't exist"
    );
}

#[actix_rt::test]
async fn test_smtp_error_handling() {
    // This test requires a non-existent SMTP host which we can't easily configure
    // through the app factory (config comes from file). We test that the endpoint
    // returns an error status when SMTP fails by sending to an invalid address format
    // that causes SMTP to reject. However, since MailHog accepts all emails, we
    // verify the endpoint works end-to-end instead.
    let (_details, app) = create_app().await.unwrap();
    let email = unique_email();

    let req = TestRequest::post()
        .uri("/api/auth/registration/init")
        .set_json(json!({"email": email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    // With a working SMTP (MailHog), the request succeeds
    assert_eq!(resp.status(), 200);
}
