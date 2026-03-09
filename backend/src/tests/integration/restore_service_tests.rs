use actix_web::test::TestRequest;
use actix_web::dev::Service;
use bcrypt;
use chrono::{Duration, Utc};
use crate::models::password_restore::PasswordRestoreSession;
use crate::tests::test_app::{create_app_with_fixtures, AppDetails};
use crate::tests::utils::unique_email;
use serde_json::{json, Value};
use uuid::Uuid;

struct UserFixture {
    email: String,
    password: String,
    user_id: Uuid,
}

async fn create_app_with_user() -> anyhow::Result<(
    AppDetails,
    impl Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
    >,
    UserFixture,
)> {
    let test_email = unique_email();
    let test_password = "SecurePass123".to_string();
    let email_clone = test_email.clone();
    let password_clone = test_password.clone();

    let (details, app, user_id) = create_app_with_fixtures(move |det| {
        let email = email_clone.clone();
        let password = password_clone.clone();
        async move {
            let password_hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST).unwrap();
            let user_id = Uuid::new_v4();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO users (id, password_hash, created_at) VALUES (?, ?, ?)",
                    rusqlite::params![&user_id, &password_hash, Utc::now()],
                )
                .unwrap();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO emails (email, user_id, is_verified) VALUES (?, ?, 1)",
                    rusqlite::params![&email, &user_id],
                )
                .unwrap();
            user_id
        }
    })
    .await?;

    Ok((
        details,
        app,
        UserFixture {
            email: test_email,
            password: test_password,
            user_id,
        },
    ))
}

fn get_restore_session_from_db(
    sqlite: &crate::providers::sqlite::SQLiteProvider,
    email: &str,
) -> Option<PasswordRestoreSession> {
    let conn = sqlite.get_connection().ok()?;
    let mut stmt = conn
        .prepare("SELECT * FROM password_restore_sessions WHERE email = ?")
        .ok()?;
    let mut rows = stmt.query(rusqlite::params![email]).ok()?;
    if let Some(row_result) = rows.next().ok()? {
        Some(PasswordRestoreSession::from_row(row_result).unwrap())
    } else {
        None
    }
}

// --- restore init ---

#[actix_rt::test]
async fn test_restore_init_successful() {
    let (_details, app, user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": user.email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["message"], "Verification email sent");
    assert!(body["resend_available_at"].as_f64().is_some());
}

#[actix_rt::test]
async fn test_restore_init_creates_session_in_database() {
    let (details, app, user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": user.email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let _body: Value = actix_web::test::read_body_json(resp).await;

    let session = get_restore_session_from_db(&details.sqlite, &user.email);
    assert!(session.is_some(), "Restore session should be created in database");

    let session = session.unwrap();
    assert_eq!(session.email, user.email);
    assert_eq!(session.user_id, user.user_id);
    assert_eq!(session.verification_code.len(), 6);
    assert!(session.verification_code.chars().all(|c| c.is_ascii_digit()));
}

#[actix_rt::test]
async fn test_restore_init_session_expires_in_15_minutes() {
    let (details, app, user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": user.email}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let _body: Value = actix_web::test::read_body_json(resp).await;

    let session = get_restore_session_from_db(&details.sqlite, &user.email).unwrap();
    let expected_expires = session.created_at + Duration::minutes(15);
    let diff = (session.expires_at.timestamp() - expected_expires.timestamp()).abs();

    assert!(diff <= 1, "Session should expire in 15 minutes");
}

#[actix_rt::test]
async fn test_restore_init_email_not_found() {
    let (_details, app, _user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": "nobody@example.com"}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 400);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["code"], "EmailNotFound");
}

#[actix_rt::test]
async fn test_restore_init_invalid_email() {
    let (_details, app, _user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": "bad-email"}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 400);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["code"], "InvalidEmail");
}

#[actix_rt::test]
async fn test_restore_init_empty_email() {
    let (_details, app, _user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": ""}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 400);
}

#[actix_rt::test]
async fn test_restore_init_reinit_replaces_session() {
    let (details, app, user) = create_app_with_user().await.unwrap();

    let req1 = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": user.email}))
        .to_request();
    let resp1 = app.call(req1).await.unwrap();
    assert_eq!(resp1.status(), 200);
    let _body1: Value = actix_web::test::read_body_json(resp1).await;

    let session1 = get_restore_session_from_db(&details.sqlite, &user.email).unwrap();

    let req2 = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": user.email}))
        .to_request();
    let resp2 = app.call(req2).await.unwrap();
    assert_eq!(resp2.status(), 200);
    let _body2: Value = actix_web::test::read_body_json(resp2).await;

    let session2 = get_restore_session_from_db(&details.sqlite, &user.email).unwrap();
    assert_ne!(session1.id, session2.id, "New session should have different ID");

    // Only one session should exist
    let conn = details.sqlite.get_connection().unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM password_restore_sessions WHERE email = ?",
            rusqlite::params![&user.email],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "Should have exactly one restore session per email");
}

// --- restore verify ---

#[actix_rt::test]
async fn test_restore_verify_successful() {
    let (details, app, user) = create_app_with_user().await.unwrap();

    // Init restore
    let req = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": user.email}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let _body: Value = actix_web::test::read_body_json(resp).await;

    // Get code from DB
    let session = get_restore_session_from_db(&details.sqlite, &user.email).unwrap();

    // Verify
    let req = TestRequest::post()
        .uri("/api/auth/password/restore/verify")
        .set_json(json!({"email": user.email, "code": session.verification_code}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert!(body["session_id"].as_str().is_some());

    let returned_id: Uuid = body["session_id"].as_str().unwrap().parse().unwrap();
    assert_eq!(returned_id, session.id);
}

#[actix_rt::test]
async fn test_restore_verify_wrong_code() {
    let (_details, app, user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": user.email}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    let _body: Value = actix_web::test::read_body_json(resp).await;

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/verify")
        .set_json(json!({"email": user.email, "code": "000000"}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 400);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["code"], "InvalidCode");
}

#[actix_rt::test]
async fn test_restore_verify_no_session() {
    let (_details, app, _user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/verify")
        .set_json(json!({"email": "nobody@example.com", "code": "123456"}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["code"], "SessionNotFound");
}

#[actix_rt::test]
async fn test_restore_verify_expired_session() {
    let test_email = unique_email();
    let email_clone = test_email.clone();

    let (_details, app, _) = create_app_with_fixtures(move |det| {
        let email = email_clone.clone();
        async move {
            let password_hash = bcrypt::hash("Pass123", bcrypt::DEFAULT_COST).unwrap();
            let user_id = Uuid::new_v4();
            let now = Utc::now();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO users (id, password_hash, created_at) VALUES (?, ?, ?)",
                    rusqlite::params![&user_id, &password_hash, now],
                )
                .unwrap();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO emails (email, user_id, is_verified) VALUES (?, ?, 1)",
                    rusqlite::params![&email, &user_id],
                )
                .unwrap();

            // Insert an already-expired restore session
            let session_id = Uuid::new_v4();
            let expired_at = now - Duration::minutes(1);
            det.sqlite
                .execute_with_params(
                    "INSERT INTO password_restore_sessions (id, user_id, email, verification_code, created_at, expires_at, resend_available_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                    rusqlite::params![&session_id, &user_id, &email, "123456", now - Duration::minutes(20), expired_at, now],
                )
                .unwrap();
        }
    })
    .await
    .unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/verify")
        .set_json(json!({"email": test_email, "code": "123456"}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["code"], "ExpiredSession");
}

// --- restore complete ---

#[actix_rt::test]
async fn test_restore_complete_successful() {
    let (details, app, user) = create_app_with_user().await.unwrap();

    // Init
    let req = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": user.email}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    let _body: Value = actix_web::test::read_body_json(resp).await;

    // Get code and verify
    let session = get_restore_session_from_db(&details.sqlite, &user.email).unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/verify")
        .set_json(json!({"email": user.email, "code": session.verification_code}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;
    let session_id = body["session_id"].as_str().unwrap();

    // Complete
    let req = TestRequest::post()
        .uri("/api/auth/password/restore/complete")
        .set_json(json!({
            "session_id": session_id,
            "password": "NewPassword456",
            "password_confirm": "NewPassword456"
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["message"], "Password changed successfully");
}

#[actix_rt::test]
async fn test_restore_complete_updates_password() {
    let (details, app, user) = create_app_with_user().await.unwrap();
    let new_password = "BrandNewPass789";

    // Init + verify + complete
    let req = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": user.email}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    let _body: Value = actix_web::test::read_body_json(resp).await;

    let session = get_restore_session_from_db(&details.sqlite, &user.email).unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/verify")
        .set_json(json!({"email": user.email, "code": session.verification_code}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;
    let session_id = body["session_id"].as_str().unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/complete")
        .set_json(json!({
            "session_id": session_id,
            "password": new_password,
            "password_confirm": new_password
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let _body: Value = actix_web::test::read_body_json(resp).await;

    // Login with new password should succeed
    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": user.email, "password": new_password}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Login with old password should fail
    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": user.email, "password": user.password}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_restore_complete_deletes_session() {
    let (details, app, user) = create_app_with_user().await.unwrap();

    // Init + verify + complete
    let req = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": user.email}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    let _body: Value = actix_web::test::read_body_json(resp).await;

    let session = get_restore_session_from_db(&details.sqlite, &user.email).unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/verify")
        .set_json(json!({"email": user.email, "code": session.verification_code}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;
    let session_id = body["session_id"].as_str().unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/complete")
        .set_json(json!({
            "session_id": session_id,
            "password": "NewPass123",
            "password_confirm": "NewPass123"
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let _body: Value = actix_web::test::read_body_json(resp).await;

    // Session should be deleted
    let session = get_restore_session_from_db(&details.sqlite, &user.email);
    assert!(session.is_none(), "Restore session should be deleted after completion");
}

#[actix_rt::test]
async fn test_restore_complete_password_mismatch() {
    let (details, app, user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/init")
        .set_json(json!({"email": user.email}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    let _body: Value = actix_web::test::read_body_json(resp).await;

    let session = get_restore_session_from_db(&details.sqlite, &user.email).unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/verify")
        .set_json(json!({"email": user.email, "code": session.verification_code}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;
    let session_id = body["session_id"].as_str().unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/complete")
        .set_json(json!({
            "session_id": session_id,
            "password": "NewPass123",
            "password_confirm": "DifferentPass456"
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 400);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["code"], "PasswordMismatch");
}

#[actix_rt::test]
async fn test_restore_complete_no_session() {
    let (_details, app, _user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/complete")
        .set_json(json!({
            "session_id": Uuid::new_v4(),
            "password": "NewPass123",
            "password_confirm": "NewPass123"
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["code"], "SessionNotFound");
}

#[actix_rt::test]
async fn test_restore_complete_expired_session() {
    let test_email = unique_email();
    let email_clone = test_email.clone();

    let (_details, app, session_id) = create_app_with_fixtures(move |det| {
        let email = email_clone.clone();
        async move {
            let password_hash = bcrypt::hash("Pass123", bcrypt::DEFAULT_COST).unwrap();
            let user_id = Uuid::new_v4();
            let now = Utc::now();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO users (id, password_hash, created_at) VALUES (?, ?, ?)",
                    rusqlite::params![&user_id, &password_hash, now],
                )
                .unwrap();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO emails (email, user_id, is_verified) VALUES (?, ?, 1)",
                    rusqlite::params![&email, &user_id],
                )
                .unwrap();

            let session_id = Uuid::new_v4();
            let expired_at = now - Duration::minutes(1);
            det.sqlite
                .execute_with_params(
                    "INSERT INTO password_restore_sessions (id, user_id, email, verification_code, created_at, expires_at, resend_available_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                    rusqlite::params![&session_id, &user_id, &email, "123456", now - Duration::minutes(20), expired_at, now],
                )
                .unwrap();
            session_id
        }
    })
    .await
    .unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/password/restore/complete")
        .set_json(json!({
            "session_id": session_id,
            "password": "NewPass123",
            "password_confirm": "NewPass123"
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["code"], "ExpiredSession");
}
