use bcrypt::hash;
use chrono::Utc;
use crate::models::auth::AuthError;
use crate::models::session::Session;
use crate::providers::sqlite::SQLiteProvider;
use crate::use_cases::login as login_use_case;
use crate::tests::utils::{temp_sqlite_path, unique_email};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use serde_json::Value;

async fn setup_test_db_with_user(
    email: &str,
    password: &str,
) -> (PathBuf, Arc<SQLiteProvider>, String, String, Uuid) {
    let temp_dir = std::env::temp_dir().join(format!("test_fs_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    
    let temp_db = PathBuf::from(temp_sqlite_path());
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
    
    let password_hash = hash(password, bcrypt::DEFAULT_COST).unwrap();
    let user_id = Uuid::new_v4();
    let created_at = Utc::now();
    
    conn.execute(
        "INSERT INTO users (id, email, password_hash, created_at) VALUES (?, ?, ?, ?)",
        rusqlite::params![&user_id, email, &password_hash, created_at],
    ).unwrap();
    
    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&sdk_config);
    
    let sqlite = Arc::new(SQLiteProvider::new_for_test(
        conn,
        temp_db.clone(),
        Arc::new(crate::providers::s3::S3Provider {
            client: s3_client,
            bucket: "test-bucket-".to_string(),
        }),
        Arc::new(crate::providers::local_filesystem::LocalFileSystemProvider::new(temp_dir).unwrap()),
        "test.db".to_string(),
    ));
    
    (temp_db, sqlite, email.to_string(), password.to_string(), user_id)
}

async fn get_session_from_db(sqlite: &SQLiteProvider, user_id: Uuid) -> Option<Session> {
    let conn = sqlite.get_connection().ok()?;
    let mut stmt = conn.prepare("SELECT * FROM sessions WHERE user_id = ?").ok()?;
    let mut rows = stmt.query(rusqlite::params![user_id]).ok()?;
    if let Some(row_result) = rows.next().ok()? {
        Some(Session::from_row(row_result).unwrap())
    } else {
        None
    }
}

#[actix_rt::test]
async fn test_login_successful_returns_valid_tokens() {
    let email = unique_email();
    let password = "SecurePass123";
    let (_temp_db, sqlite, email, password, _user_id) = setup_test_db_with_user(&email, &password).await;
    let jwt_secret = "test-secret-key-for-jwt-signing";
    
    let response = login_use_case(sqlite, &email, &password, &jwt_secret).await;
    
    assert!(response.is_ok(), "Login should succeed: {:?}", response.err());
    
    let result = response.unwrap();
    assert_eq!(result.status, "ok");
    assert_eq!(result.user.email, email);
    assert!(!result.access_token.is_empty());
    assert!(!result.refresh_token.is_empty());
}

#[actix_rt::test]
async fn test_tokens_are_valid_jwt_format() {
    let email = unique_email();
    let password = "SecurePass123";
    let (_temp_db, sqlite, email, password, _user_id) = setup_test_db_with_user(&email, &password).await;
    let jwt_secret = "test-secret-key-for-jwt-signing";
    
    let response = login_use_case(sqlite, &email, &password, &jwt_secret).await.unwrap();
    
    let access_parts: Vec<&str> = response.access_token.split('.').collect();
    let refresh_parts: Vec<&str> = response.refresh_token.split('.').collect();
    
    assert_eq!(access_parts.len(), 3, "Access token should have 3 parts separated by dots");
    assert_eq!(refresh_parts.len(), 3, "Refresh token should have 3 parts separated by dots");
    
    assert!(!access_parts[0].is_empty(), "JWT header should not be empty");
    assert!(!access_parts[1].is_empty(), "JWT payload should not be empty");
    assert!(!access_parts[2].is_empty(), "JWT signature should not be empty");
}

#[actix_rt::test]
async fn test_tokens_can_be_decoded_with_expected_claims() {
    let email = unique_email();
    let password = "SecurePass123";
    let (_temp_db, sqlite, email, password, _user_id) = setup_test_db_with_user(&email, &password).await;
    let jwt_secret = "test-secret-key-for-jwt-signing";
    
    let response = login_use_case(sqlite.clone(), &email, &password, &jwt_secret).await.unwrap();
    
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
    
    let access_token_data: jsonwebtoken::TokenData<Value> = decode(&response.access_token, &decoding_key, &validation).unwrap();
    let refresh_token_data: jsonwebtoken::TokenData<Value> = decode(&response.refresh_token, &decoding_key, &validation).unwrap();
    
    let now = Utc::now().timestamp();
    let access_exp = access_token_data.claims.get("exp").unwrap().as_u64().unwrap() as i64;
    let refresh_exp = refresh_token_data.claims.get("exp").unwrap().as_u64().unwrap() as i64;
    
    assert!(access_exp > now, "Access token should not be expired yet");
    assert!(refresh_exp > now, "Refresh token should not be expired yet");
    
    assert!(refresh_exp > access_exp, "Refresh token should expire after access token");
    
    let access_hours = (access_exp - now) / 3600;
    let refresh_days = (refresh_exp - now) / 86400;
    
    assert!(access_hours <= 24, "Access token should be short-lived (less than 24 hours)");
    assert!(refresh_days >= 1, "Refresh token should be long-lived (at least 1 day)");
}

#[actix_rt::test]
async fn test_session_user_id_matches_logged_in_user() {
    let email = unique_email();
    let password = "SecurePass123";
    let (_temp_db, sqlite, email, password, user_id) = setup_test_db_with_user(&email, &password).await;
    let jwt_secret = "test-secret-key-for-jwt-signing";
    
    let response = login_use_case(sqlite.clone(), &email, &password, &jwt_secret).await.unwrap();
    
    let session = get_session_from_db(&sqlite, user_id).await.unwrap();
    
    assert_eq!(session.user_id, user_id, "Session user_id should match logged-in user");
    assert_eq!(response.user.id, user_id, "Response user id should match logged-in user");
}

#[actix_rt::test]
async fn test_session_tokens_match_response() {
    let email = unique_email();
    let password = "SecurePass123";
    let (_temp_db, sqlite, email, password, user_id) = setup_test_db_with_user(&email, &password).await;
    let jwt_secret = "test-secret-key-for-jwt-signing";
    
    let response = login_use_case(sqlite.clone(), &email, &password, &jwt_secret).await.unwrap();
    
    let session = get_session_from_db(&sqlite, user_id).await.unwrap();
    
    assert_eq!(session.access_token, response.access_token, "Session access_token should match response");
    assert_eq!(session.refresh_token, response.refresh_token, "Session refresh_token should match response");
}

#[actix_rt::test]
async fn test_session_timestamps_are_set_correctly() {
    let email = unique_email();
    let password = "SecurePass123";
    let (_temp_db, sqlite, email, password, user_id) = setup_test_db_with_user(&email, &password).await;
    let jwt_secret = "test-secret-key-for-jwt-signing";
    
    let before_login = Utc::now();
    let _response = login_use_case(sqlite.clone(), &email, &password, &jwt_secret).await.unwrap();
    let after_login = Utc::now();
    
    let session = get_session_from_db(&sqlite, user_id).await.unwrap();
    
    assert!(session.created_at >= before_login, "Session created_at should be after login started");
    assert!(session.created_at <= after_login, "Session created_at should be before login finished");
    assert!(session.expires_at > session.created_at, "Session expires_at should be after created_at");
}

#[actix_rt::test]
async fn test_bcrypt_verification_failure_handling() {
    let email = unique_email();
    let password = "SecurePass123";
    let (_temp_db, sqlite, email, _password, _user_id) = setup_test_db_with_user(&email, &password).await;
    let jwt_secret = "test-secret-key-for-jwt-signing";
    
    let response = login_use_case(sqlite, &email, "different_password", &jwt_secret).await;
    
    assert!(response.is_err());
    match response {
        Err(AuthError::InvalidCredentials) => {},
        Err(e) => panic!("Expected InvalidCredentials for wrong password, got: {}", e),
        _ => panic!("Expected error"),
    }
}

#[actix_rt::test]
async fn test_database_query_failure_handling() {
    let email = unique_email();
    let password = "SecurePass123";
    let (temp_db, _sqlite, _email, _password, _user_id) = setup_test_db_with_user(&email, &password).await;
    
    let conn = rusqlite::Connection::open(&temp_db).unwrap();
    conn.execute("DROP TABLE sessions", []).unwrap();
    
    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&sdk_config);
    
    let sqlite = Arc::new(SQLiteProvider::new_for_test(
        conn,
        temp_db.clone(),
        Arc::new(crate::providers::s3::S3Provider {
            client: s3_client,
            bucket: "test-bucket-".to_string(),
        }),
        Arc::new(crate::providers::local_filesystem::LocalFileSystemProvider::new(std::env::temp_dir().join(format!("test_fs_{}", Uuid::new_v4()))).unwrap()),
        "test.db".to_string(),
    ));
    
    let jwt_secret = "test-secret-key-for-jwt-signing";
    let response = login_use_case(sqlite, &email, &password, &jwt_secret).await;
    
    assert!(response.is_err(), "Login should fail when sessions table doesn't exist");
}

#[actix_rt::test]
async fn test_jwt_signing_error_handling() {
    let email = unique_email();
    let password = "SecurePass123";
    let (_temp_db, sqlite, email, password, _user_id) = setup_test_db_with_user(&email, &password).await;
    let jwt_secret = "test-secret-key-for-jwt-signing";
    
    let response = login_use_case(sqlite, &email, &password, &jwt_secret).await;
    
    assert!(response.is_ok(), "Login should succeed with valid JWT secret");
    let result = response.unwrap();
    assert!(!result.access_token.is_empty(), "Access token should not be empty");
    assert!(!result.refresh_token.is_empty(), "Refresh token should not be empty");
}

#[actix_rt::test]
async fn test_multiple_users_isolated() {
    let email1 = unique_email();
    let email2 = unique_email();
    let password1 = "Password123";
    let password2 = "Password456";
    
    let (_temp_db, sqlite, email1, password1, user_id1) = setup_test_db_with_user(&email1, &password1).await;
    
    let conn = sqlite.get_connection().unwrap();
    let password_hash2 = hash(password2, bcrypt::DEFAULT_COST).unwrap();
    let user_id2 = Uuid::new_v4();
    let created_at = Utc::now();
    
    conn.execute(
        "INSERT INTO users (id, email, password_hash, created_at) VALUES (?, ?, ?, ?)",
        rusqlite::params![&user_id2, &email2, &password_hash2, created_at],
    ).unwrap();
    
    drop(conn);
    
    let jwt_secret = "test-secret-key-for-jwt-signing";
    
    let response1 = login_use_case(sqlite.clone(), &email1, &password1, &jwt_secret).await.unwrap();
    let response2 = login_use_case(sqlite.clone(), &email2, &password2, &jwt_secret).await.unwrap();
    
    assert_eq!(response1.user.email, email1);
    assert_eq!(response2.user.email, email2);
    assert_ne!(response1.user.id, response2.user.id);
    
    let session1 = get_session_from_db(&sqlite, user_id1).await.unwrap();
    let session2 = get_session_from_db(&sqlite, user_id2).await.unwrap();
    
    assert_eq!(session1.user_id, user_id1);
    assert_eq!(session2.user_id, user_id2);
    assert_ne!(session1.access_token, session2.access_token);
}