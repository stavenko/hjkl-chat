use actix_web::test::TestRequest;
use actix_web::dev::Service;
use bcrypt::hash;
use chrono::Utc;
use crate::models::session::Session;
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
            let password_hash = hash(&password, bcrypt::DEFAULT_COST).unwrap();
            let user_id = Uuid::new_v4();
            let created_at = Utc::now();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO users (id, password_hash, created_at) VALUES (?, ?, ?)",
                    rusqlite::params![&user_id, &password_hash, created_at],
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

fn get_session_from_db(sqlite: &crate::providers::sqlite::SQLiteProvider, user_id: Uuid) -> Option<Session> {
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
async fn test_login_successful_returns_valid_token() {
    let (_details, app, user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": user.email, "password": user.password}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["user"]["email"], user.email);
    assert!(body["token"].as_str().map(|s| !s.is_empty()).unwrap_or(false));
}

#[actix_rt::test]
async fn test_token_is_long_random_string() {
    let (_details, app, user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": user.email, "password": user.password}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;

    let token = body["token"].as_str().unwrap();
    assert!(token.len() >= 64, "Token should be at least 64 characters long");
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()), "Token should be hex-encoded");
}

#[actix_rt::test]
async fn test_session_user_id_matches_logged_in_user() {
    let (details, app, user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": user.email, "password": user.password}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;

    let response_user_id: Uuid = serde_json::from_value(body["user"]["id"].clone()).unwrap();
    let session = get_session_from_db(&details.sqlite, user.user_id).unwrap();

    assert_eq!(session.user_id, user.user_id, "Session user_id should match logged-in user");
    assert_eq!(response_user_id, user.user_id, "Response user id should match logged-in user");
}

#[actix_rt::test]
async fn test_session_token_matches_response() {
    let (details, app, user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": user.email, "password": user.password}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;

    let session = get_session_from_db(&details.sqlite, user.user_id).unwrap();

    assert_eq!(
        session.token,
        body["token"].as_str().unwrap(),
        "Session token should match response"
    );
}

#[actix_rt::test]
async fn test_session_timestamps_are_set_correctly() {
    let (details, app, user) = create_app_with_user().await.unwrap();

    let before_login = Utc::now();

    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": user.email, "password": user.password}))
        .to_request();

    let _resp = app.call(req).await.unwrap();
    let after_login = Utc::now();

    let session = get_session_from_db(&details.sqlite, user.user_id).unwrap();

    assert!(session.created_at >= before_login, "Session created_at should be after login started");
    assert!(session.created_at <= after_login, "Session created_at should be before login finished");
    assert!(session.expires_at > session.created_at, "Session expires_at should be after created_at");
}

#[actix_rt::test]
async fn test_bcrypt_verification_failure_handling() {
    let (_details, app, user) = create_app_with_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": user.email, "password": "different_password"}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_database_query_failure_handling() {
    let (details, app, _) = create_app_with_fixtures(|det| async move {
        let conn = det.sqlite.get_connection().unwrap();
        conn.execute("DROP TABLE sessions", []).unwrap();
    })
    .await
    .unwrap();

    let email = unique_email();
    let password = "SecurePass123";
    let password_hash = hash(password, bcrypt::DEFAULT_COST).unwrap();
    let user_id = Uuid::new_v4();
    details
        .sqlite
        .execute_with_params(
            "INSERT INTO users (id, password_hash, created_at) VALUES (?, ?, ?)",
            rusqlite::params![&user_id, &password_hash, Utc::now()],
        )
        .unwrap();
    details
        .sqlite
        .execute_with_params(
            "INSERT INTO emails (email, user_id, is_verified) VALUES (?, ?, 1)",
            rusqlite::params![&email, &user_id],
        )
        .unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": email, "password": password}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 500, "Login should fail when sessions table doesn't exist");
}

#[actix_rt::test]
async fn test_multiple_users_isolated() {
    let email1 = unique_email();
    let email2 = unique_email();
    let password1 = "Password123";
    let password2 = "Password456";

    let email1_clone = email1.clone();
    let email2_clone = email2.clone();

    let (details, app, (user_id1, user_id2)) = create_app_with_fixtures(move |det| {
        let e1 = email1_clone.clone();
        let e2 = email2_clone.clone();
        async move {
            let hash1 = hash(password1, bcrypt::DEFAULT_COST).unwrap();
            let hash2 = hash(password2, bcrypt::DEFAULT_COST).unwrap();
            let uid1 = Uuid::new_v4();
            let uid2 = Uuid::new_v4();
            let now = Utc::now();

            det.sqlite
                .execute_with_params(
                    "INSERT INTO users (id, password_hash, created_at) VALUES (?, ?, ?)",
                    rusqlite::params![&uid1, &hash1, now],
                )
                .unwrap();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO emails (email, user_id, is_verified) VALUES (?, ?, 1)",
                    rusqlite::params![&e1, &uid1],
                )
                .unwrap();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO users (id, password_hash, created_at) VALUES (?, ?, ?)",
                    rusqlite::params![&uid2, &hash2, now],
                )
                .unwrap();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO emails (email, user_id, is_verified) VALUES (?, ?, 1)",
                    rusqlite::params![&e2, &uid2],
                )
                .unwrap();

            (uid1, uid2)
        }
    })
    .await
    .unwrap();

    let req1 = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": email1, "password": password1}))
        .to_request();
    let resp1 = app.call(req1).await.unwrap();
    let body1: Value = actix_web::test::read_body_json(resp1).await;

    let req2 = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": email2, "password": password2}))
        .to_request();
    let resp2 = app.call(req2).await.unwrap();
    let body2: Value = actix_web::test::read_body_json(resp2).await;

    assert_eq!(body1["user"]["email"], email1);
    assert_eq!(body2["user"]["email"], email2);
    assert_ne!(body1["user"]["id"], body2["user"]["id"]);

    let session1 = get_session_from_db(&details.sqlite, user_id1).unwrap();
    let session2 = get_session_from_db(&details.sqlite, user_id2).unwrap();

    assert_eq!(session1.user_id, user_id1);
    assert_eq!(session2.user_id, user_id2);
    assert_ne!(session1.token, session2.token);
}
