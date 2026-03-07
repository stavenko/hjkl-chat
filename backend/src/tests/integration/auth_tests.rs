use bcrypt::hash;
use chrono::Utc;
use crate::models::auth::AuthError;
use crate::providers::sqlite::SQLiteProvider;
use crate::use_cases::login as login_use_case;
use crate::tests::utils::temp_sqlite_path;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

async fn setup_test_db() -> (PathBuf, Arc<SQLiteProvider>, String, String) {
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
    
    let test_email = "test@example.com";
    let test_password = "SecurePass123";
    let password_hash = hash(test_password, bcrypt::DEFAULT_COST).unwrap();
    let user_id = Uuid::new_v4();
    let created_at = Utc::now();
    
    conn.execute(
        "INSERT INTO users (id, email, password_hash, created_at) VALUES (?, ?, ?, ?)",
        rusqlite::params![&user_id, test_email, &password_hash, created_at],
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
    
    (temp_db, sqlite, test_email.to_string(), test_password.to_string())
}

#[actix_rt::test]
async fn test_login_successful() {
    let (_temp_db, sqlite, email, password) = setup_test_db().await;
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
async fn test_login_wrong_password() {
    let (_temp_db, sqlite, email, _password) = setup_test_db().await;
    let jwt_secret = "test-secret-key-for-jwt-signing";
    
    let response = login_use_case(sqlite, &email, "wrong_password", &jwt_secret).await;
    
    assert!(response.is_err(), "Login should fail with wrong password");
    
    match response {
        Err(AuthError::InvalidCredentials | AuthError::UserNotFound) => {},
        Err(e) => panic!("Expected InvalidCredentials or UserNotFound, got: {}", e),
        _ => panic!("Expected error"),
    }
}

#[actix_rt::test]
async fn test_login_nonexistent_email() {
    let (_temp_db, sqlite, _email, _password) = setup_test_db().await;
    let jwt_secret = "test-secret-key-for-jwt-signing";
    
    let response = login_use_case(sqlite, "nonexistent@example.com", "SecurePass123", &jwt_secret).await;
    
    assert!(response.is_err(), "Login should fail with non-existent email");
    
    match response {
        Err(AuthError::UserNotFound) => {},
        Err(e) => panic!("Expected UserNotFound, got: {}", e),
        _ => panic!("Expected error"),
    }
}

#[actix_rt::test]
async fn test_login_missing_fields_returns_error() {
    let (_temp_db, sqlite, _email, _password) = setup_test_db().await;
    let jwt_secret = "test-secret-key-for-jwt-signing";
    
    let response = login_use_case(sqlite, "", "password", &jwt_secret).await;
    
    assert!(response.is_err(), "Login should fail with empty email");
}