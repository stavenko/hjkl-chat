use chrono::{Duration, Utc};
use crate::models::registration::{RegistrationError, RegistrationSession};
use crate::providers::sqlite::SQLiteProvider;
use crate::providers::smtp::{SMTPProvider, SmtpClient, SMTPProviderResult};
use crate::use_cases::registration::RegistrationUseCase;
use crate::tests::utils::{temp_sqlite_path, unique_email};
use crate::tests::integration::mailhog_tests::get_mailhog_messages;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;
use async_trait::async_trait;

pub struct MockSMTPProvider {
    pub emails_sent: Arc<parking_lot::Mutex<Vec<(String, String, String)>>>,
}

impl MockSMTPProvider {
    pub fn new() -> Self {
        Self {
            emails_sent: Arc::new(parking_lot::Mutex::new(Vec::new())),
        }
    }

    #[allow(dead_code)]
    pub fn get_emails(&self) -> Vec<(String, String, String)> {
        self.emails_sent.lock().clone()
    }

    #[allow(dead_code)]
    pub fn clear_emails(&self) {
        self.emails_sent.lock().clear();
    }
}

#[async_trait]
impl SmtpClient for MockSMTPProvider {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> SMTPProviderResult<()> {
        self.emails_sent.lock().push((to.to_string(), subject.to_string(), body.to_string()));
        Ok(())
    }
}

async fn setup_test_db_with_tables(temp_db: PathBuf) -> Arc<SQLiteProvider> {
    let conn = rusqlite::Connection::open(&temp_db).unwrap();
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id UUID PRIMARY KEY,
            user_id UUID REFERENCES users(id),
            access_token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS registration_sessions (
            id UUID PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            verification_code TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            resend_available_at TIMESTAMP NOT NULL
        )",
        [],
    ).unwrap();
    
    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&sdk_config);
    
    let temp_dir = std::env::temp_dir().join(format!("test_fs_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    
    Arc::new(SQLiteProvider::new_for_test(
        conn,
        temp_db.clone(),
        Arc::new(crate::providers::s3::S3Provider {
            client: s3_client,
            bucket: "test-bucket-".to_string(),
        }),
        Arc::new(crate::providers::local_filesystem::LocalFileSystemProvider::new(temp_dir).unwrap()),
        "test.db".to_string(),
    ))
}

#[allow(dead_code)]
async fn setup_test_use_case_with_mock(temp_db: PathBuf) -> (Arc<SQLiteProvider>, RegistrationUseCase<MockSMTPProvider>, Arc<MockSMTPProvider>) {
    let temp_dir = std::env::temp_dir().join(format!("test_fs_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    
    let conn = rusqlite::Connection::open(&temp_db).unwrap();
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id UUID PRIMARY KEY,
            user_id UUID REFERENCES users(id),
            access_token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS registration_sessions (
            id UUID PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            verification_code TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            resend_available_at TIMESTAMP NOT NULL
        )",
        [],
    ).unwrap();
    
    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&sdk_config);
    
    let db_path = temp_dir.join("test.db");
    std::fs::write(&db_path, vec![]).unwrap();
    
    let sqlite = Arc::new(SQLiteProvider::new_for_test(
        conn,
        db_path,
        Arc::new(crate::providers::s3::S3Provider {
            client: s3_client,
            bucket: "test-bucket-".to_string(),
        }),
        Arc::new(crate::providers::local_filesystem::LocalFileSystemProvider::new(temp_dir).unwrap()),
        "test.db".to_string(),
    ));
    
    let smtp = Arc::new(MockSMTPProvider::new());
    let use_case = RegistrationUseCase::new_for_test(sqlite.clone(), smtp.clone());
    
    (sqlite, use_case, smtp)
}

#[allow(dead_code)]
async fn setup_test_use_case_with_real_smtp(temp_db: PathBuf, smtp_host: &str) -> (Arc<SQLiteProvider>, RegistrationUseCase<SMTPProvider>) {
    let temp_dir = std::env::temp_dir().join(format!("test_fs_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    
    let conn = rusqlite::Connection::open(&temp_db).unwrap();
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id UUID PRIMARY KEY,
            user_id UUID REFERENCES users(id),
            access_token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS registration_sessions (
            id UUID PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            verification_code TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            resend_available_at TIMESTAMP NOT NULL
        )",
        [],
    ).unwrap();
    
    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&sdk_config);
    
    let db_path = temp_dir.join("test.db");
    std::fs::write(&db_path, vec![]).unwrap();
    
    let sqlite = Arc::new(SQLiteProvider::new_for_test(
        conn,
        db_path,
        Arc::new(crate::providers::s3::S3Provider {
            client: s3_client,
            bucket: "test-bucket-".to_string(),
        }),
        Arc::new(crate::providers::local_filesystem::LocalFileSystemProvider::new(temp_dir).unwrap()),
        "test.db".to_string(),
    ));
    
    let smtp = Arc::new(SMTPProvider::new(
        smtp_host,
        1025,
        false,
        "test",
        "test",
        "noreply@example.com",
    ).unwrap());
    
    let use_case = RegistrationUseCase::new(sqlite.clone(), smtp);
    
    (sqlite, use_case)
}

fn get_session_from_db(sqlite: &SQLiteProvider, session_id: Uuid) -> Option<RegistrationSession> {
    let conn = sqlite.get_connection().ok()?;
    let mut stmt = conn.prepare("SELECT * FROM registration_sessions WHERE id = ?").ok()?;
    let mut rows = stmt.query(rusqlite::params![session_id]).ok()?;
    if let Some(row_result) = rows.next().ok()? {
        Some(RegistrationSession::from_row(row_result).unwrap())
    } else {
        None
    }
}

#[allow(dead_code)]
fn get_session_by_email(sqlite: &SQLiteProvider, email: &str) -> Option<RegistrationSession> {
    let conn = sqlite.get_connection().ok()?;
    let mut stmt = conn.prepare("SELECT * FROM registration_sessions WHERE email = ?").ok()?;
    let mut rows = stmt.query(rusqlite::params![email]).ok()?;
    if let Some(row_result) = rows.next().ok()? {
        Some(RegistrationSession::from_row(row_result).unwrap())
    } else {
        None
    }
}

async fn setup_test_use_case(
    temp_db: PathBuf,
    _smtp_host: Option<&str>,
) -> (Arc<SQLiteProvider>, RegistrationUseCase<MockSMTPProvider>) {
    let temp_dir = std::env::temp_dir().join(format!("test_fs_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let conn = rusqlite::Connection::open(&temp_db).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id UUID PRIMARY KEY,
            user_id UUID REFERENCES users(id),
            access_token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS registration_sessions (
            id UUID PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            verification_code TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            resend_available_at TIMESTAMP NOT NULL
        )",
        [],
    ).unwrap();

    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&sdk_config);

    let db_path = temp_dir.join("test.db");
    std::fs::write(&db_path, vec![]).unwrap();

    let sqlite = Arc::new(SQLiteProvider::new_for_test(
        conn,
        db_path,
        Arc::new(crate::providers::s3::S3Provider {
            client: s3_client,
            bucket: "test-bucket-".to_string(),
        }),
        Arc::new(crate::providers::local_filesystem::LocalFileSystemProvider::new(temp_dir).unwrap()),
        "test.db".to_string(),
    ));

    let smtp = Arc::new(MockSMTPProvider::new());
    let use_case = RegistrationUseCase::new_for_test(sqlite.clone(), smtp.clone());

    (sqlite, use_case)
}

#[actix_rt::test]
async fn test_registration_init_successful() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();

    let (_sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;

    let response = use_case.init_registration(&email).await;

    assert!(response.is_ok(), "Registration init should succeed: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.status, "ok");
    assert_eq!(result.message, "Verification email sent");
    assert!(!result.session_id.is_nil());
    assert!(result.resend_available_at > Utc::now());
}

#[actix_rt::test]
async fn test_registration_init_generates_6_digit_code() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let (sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let response = use_case.init_registration(&email).await.unwrap();
    let session = get_session_from_db(&sqlite, response.session_id).unwrap();
    
    assert_eq!(session.verification_code.len(), 6, "Verification code should be exactly 6 digits");
    assert!(session.verification_code.chars().all(|c| c.is_ascii_digit()), "Verification code should contain only digits");
}

#[actix_rt::test]
async fn test_registration_init_creates_session_in_database() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let (sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let response = use_case.init_registration(&email).await.unwrap();
    
    let session = get_session_from_db(&sqlite, response.session_id);
    
    assert!(session.is_some(), "Session should be created in database");
    
    let session = session.unwrap();
    assert_eq!(session.id, response.session_id);
    assert_eq!(session.email, email);
    assert!(!session.verification_code.is_empty());
    assert!(session.created_at <= Utc::now());
    assert!(session.expires_at > session.created_at);
    assert!(session.resend_available_at > session.created_at);
}

#[actix_rt::test]
#[ignore = "Requires MailHog service running at localhost:1025"]
async fn test_registration_init_sends_email_via_smtp() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let (_sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let response = use_case.init_registration(&email).await;
    
    assert!(response.is_ok(), "Email should be sent successfully: {:?}", response.err());
}

#[actix_rt::test]
async fn test_registration_init_session_expires_in_15_minutes() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let (sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let response = use_case.init_registration(&email).await.unwrap();
    let session = get_session_from_db(&sqlite, response.session_id).unwrap();
    
    let expected_expires = session.created_at + Duration::minutes(15);
    let diff = (session.expires_at.timestamp() - expected_expires.timestamp()).abs();
    
    assert!(diff <= 1, "Session should expire in 15 minutes (allow 1 second tolerance), got: {:?}", session.expires_at);
}

#[actix_rt::test]
async fn test_registration_init_resend_available_in_60_seconds() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let (sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let response = use_case.init_registration(&email).await.unwrap();
    let session = get_session_from_db(&sqlite, response.session_id).unwrap();
    
    let expected_resend = session.created_at + Duration::seconds(60);
    let diff = (session.resend_available_at.timestamp() - expected_resend.timestamp()).abs();
    
    assert!(diff <= 1, "Resend should be available in 60 seconds (allow 1 second tolerance), got: {:?}", session.resend_available_at);
    
    assert_eq!(response.resend_available_at, session.resend_available_at);
}

#[actix_rt::test]
async fn test_registration_init_empty_email_returns_error() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = "";
    
    let (_sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let response = use_case.init_registration(email).await;
    
    assert!(response.is_err(), "Empty email should return error");
    
    match response {
        Err(RegistrationError::InvalidEmail) => {},
        Err(e) => panic!("Expected InvalidEmail error, got: {}", e),
        _ => panic!("Expected error for empty email"),
    }
}

#[actix_rt::test]
async fn test_registration_init_invalid_email_format_returns_error() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let invalid_emails = vec![
        "invalid-email",
        "missing@domain",
        "no-at-sign.com",
    ];
    
    for invalid_email in invalid_emails {
        let (_sqlite, use_case) = setup_test_use_case(temp_db.clone(), Some("localhost")).await;
        
        let response = use_case.init_registration(invalid_email).await;
        
        assert!(response.is_err(), "Invalid email '{}' should return error", invalid_email);
    }
}

#[actix_rt::test]
async fn test_registration_init_duplicate_email_returns_error() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let (_sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let _first_response = use_case.init_registration(&email).await.unwrap();
    
    let second_response = use_case.init_registration(&email).await;
    
    assert!(second_response.is_err(), "Duplicate email should return error");
    
    match second_response {
        Err(RegistrationError::DatabaseError(_)) => {},
        Err(RegistrationError::DatabaseProvider(_)) => {},
        Err(RegistrationError::SessionAlreadyExists) => {},
        Err(e) => panic!("Expected database error for duplicate email, got: {}", e),
        _ => panic!("Expected error for duplicate email"),
    }
}

#[actix_rt::test]
async fn test_session_stored_with_correct_schema() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let (sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let response = use_case.init_registration(&email).await.unwrap();
    let session = get_session_from_db(&sqlite, response.session_id).unwrap();
    
    assert_eq!(session.id, response.session_id);
    assert_eq!(session.email, email);
    assert_eq!(session.verification_code.len(), 6);
    assert!(session.created_at <= Utc::now());
    assert!(session.expires_at > session.created_at);
    assert!(session.resend_available_at > session.created_at);
    
    let conn = sqlite.get_connection().unwrap();
    let mut stmt = conn.prepare("PRAGMA table_info(registration_sessions)").unwrap();
    let columns = stmt.query_map([], |row| {
        Ok((row.get::<_, String>("name")?, row.get::<_, String>("type")?))
    }).unwrap();
    
    let column_names: Vec<String> = columns.filter_map(|r| r.ok().map(|(name, _)| name)).collect();
    
    assert!(column_names.contains(&"id".to_string()), "Table should have 'id' column");
    assert!(column_names.contains(&"email".to_string()), "Table should have 'email' column");
    assert!(column_names.contains(&"verification_code".to_string()), "Table should have 'verification_code' column");
    assert!(column_names.contains(&"created_at".to_string()), "Table should have 'created_at' column");
    assert!(column_names.contains(&"expires_at".to_string()), "Table should have 'expires_at' column");
    assert!(column_names.contains(&"resend_available_at".to_string()), "Table should have 'resend_available_at' column");
}

#[actix_rt::test]
async fn test_session_uuid_is_valid_format() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let (sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let response = use_case.init_registration(&email).await.unwrap();
    let session = get_session_from_db(&sqlite, response.session_id).unwrap();
    
    let uuid_str = session.id.to_string();
    assert!(uuid::Uuid::parse_str(&uuid_str).is_ok(), "Session ID should be valid UUID");
    assert_eq!(uuid_str.len(), 36, "UUID string should be 36 characters with dashes");
}

#[actix_rt::test]
async fn test_session_email_is_unique() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let (_sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let _first = use_case.init_registration(&email).await.unwrap();
    let second = use_case.init_registration(&email).await;
    
    assert!(second.is_err(), "Second registration with same email should fail due to UNIQUE constraint");
}

#[actix_rt::test]
async fn test_session_timestamps_are_rfc3339_compatible() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let (sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let response = use_case.init_registration(&email).await.unwrap();
    let session = get_session_from_db(&sqlite, response.session_id).unwrap();
    
    let created_at_str = session.created_at.to_rfc3339();
    let expires_at_str = session.expires_at.to_rfc3339();
    let resend_str = session.resend_available_at.to_rfc3339();
    
    assert!(chrono::DateTime::parse_from_rfc3339(&created_at_str).is_ok(), "created_at should be RFC3339 compatible");
    assert!(chrono::DateTime::parse_from_rfc3339(&expires_at_str).is_ok(), "expires_at should be RFC3339 compatible");
    assert!(chrono::DateTime::parse_from_rfc3339(&resend_str).is_ok(), "resend_available_at should be RFC3339 compatible");
}

#[actix_rt::test]
#[ignore = "Requires MailHog service running at localhost:1025 and API at localhost:8025"]
async fn test_email_contains_verification_code() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let (sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let response = use_case.init_registration(&email).await.unwrap();
    let session = get_session_from_db(&sqlite, response.session_id).unwrap();
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    let messages = get_mailhog_messages(&email).await;
    
    if let Ok(msgs) = messages {
        assert!(!msgs.is_empty(), "Should have sent email to MailHog");
        
        let message = msgs.first().unwrap();
        let content = message.get("Body").and_then(|b| b.get("Raw")).and_then(|r| r.as_str());
        
        assert!(content.is_some(), "Email should have body content");
        assert!(
            content.unwrap().contains(&session.verification_code),
            "Email body should contain verification code: {}",
            session.verification_code
        );
    }
}

#[actix_rt::test]
#[ignore = "Requires MailHog service running at localhost:1025 and API at localhost:8025"]
async fn test_email_sent_to_correct_address() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let (_sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let _response = use_case.init_registration(&email).await.unwrap();
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    let messages = get_mailhog_messages(&email).await;
    
    if let Ok(msgs) = messages {
        assert!(!msgs.is_empty(), "Should have sent email");
        
        let message = msgs.first().unwrap();
        let to = message.get("To").and_then(|t| t.as_str());
        
        assert!(to.is_some(), "Email should have 'To' field");
        assert!(
            to.unwrap().contains(&email),
            "Email should be sent to correct address: {}, got: {}",
            email,
            to.unwrap()
        );
    }
}

#[actix_rt::test]
#[ignore = "Requires MailHog service running at localhost:1025 and API at localhost:8025"]
async fn test_email_has_subject_line() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let (_sqlite, use_case) = setup_test_use_case(temp_db, Some("localhost")).await;
    
    let _response = use_case.init_registration(&email).await.unwrap();
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    let messages = get_mailhog_messages(&email).await;
    
    if let Ok(msgs) = messages {
        assert!(!msgs.is_empty(), "Should have sent email");
        
        let message = msgs.first().unwrap();
        let subject = message.get("Subject").and_then(|s| s.as_str());
        
        assert!(subject.is_some(), "Email should have subject");
        assert_eq!(
            subject.unwrap(),
            "Your Registration Verification Code",
            "Email should have correct subject line"
        );
    }
}

#[actix_rt::test]
async fn test_database_error_handling() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let sqlite = setup_test_db_with_tables(temp_db.clone()).await;
    
    let conn = sqlite.get_connection().unwrap();
    conn.execute("DROP TABLE registration_sessions", []).unwrap();
    drop(conn);
    
    let smtp = Arc::new(SMTPProvider::new(
        "localhost",
        1025,
        false,
        "test",
        "test",
        "noreply@example.com",
    ).unwrap_or_else(|_| {
        SMTPProvider::new(
            "localhost",
            1025,
            false,
            "test",
            "test",
            "noreply@example.com",
        ).unwrap()
    }));
    
    let use_case = RegistrationUseCase::new(sqlite, smtp);
    
    let response = use_case.init_registration(&email).await;
    
    assert!(response.is_err(), "Should fail with database error when table doesn't exist");
    
    match response {
        Err(RegistrationError::DatabaseError(_)) => {},
        Err(RegistrationError::DatabaseProvider(_)) => {},
        Err(e) => panic!("Expected DatabaseError, got: {}", e),
        _ => panic!("Expected error"),
    }
}

#[actix_rt::test]
async fn test_smtp_error_handling() {
    let temp_db = PathBuf::from(temp_sqlite_path());
    let email = unique_email();
    
    let sqlite = setup_test_db_with_tables(temp_db).await;
    
    let smtp = Arc::new(SMTPProvider::new(
        "nonexistent-host",
        9999,
        false,
        "test",
        "test",
        "noreply@example.com",
    ).unwrap_or_else(|_| {
        SMTPProvider::new(
            "nonexistent-host",
            9999,
            false,
            "test",
            "test",
            "noreply@example.com",
        ).unwrap()
    }));
    
    let use_case = RegistrationUseCase::new(sqlite, smtp);
    
    let response = use_case.init_registration(&email).await;
    
    assert!(response.is_err(), "Should fail with SMTP error when host is unreachable");
    
    match response {
        Err(RegistrationError::SmtpError(_)) => {},
        Err(RegistrationError::DatabaseError(_)) => {},
        Err(e) => panic!("Expected SmtpError or DatabaseError, got: {}", e),
        _ => panic!("Expected error"),
    }
}