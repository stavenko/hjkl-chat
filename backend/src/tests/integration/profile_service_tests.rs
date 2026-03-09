use actix_web::test::TestRequest;
use actix_web::dev::Service;
use bcrypt;
use chrono::{Duration, Utc};
use crate::tests::test_app::{create_app_with_fixtures, AppDetails};
use crate::tests::utils::unique_email;
use serde_json::{json, Value};
use uuid::Uuid;

struct UserFixture {
    email: String,
    password: String,
    user_id: Uuid,
    token: String,
}

async fn create_app_with_logged_in_user() -> anyhow::Result<(
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

    let (details, app, (user_id, token)) = create_app_with_fixtures(move |det| {
        let email = email_clone.clone();
        let password = password_clone.clone();
        async move {
            let password_hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST).unwrap();
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

            // Create a session directly
            let token = "a".repeat(128);
            let session_id = Uuid::new_v4();
            let expires_at = now + Duration::days(30);
            det.sqlite
                .execute_with_params(
                    "INSERT INTO sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
                    rusqlite::params![&session_id, &user_id, &token, &expires_at, &now],
                )
                .unwrap();

            (user_id, token)
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
            token,
        },
    ))
}

async fn create_app_with_user_with_profile() -> anyhow::Result<(
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

    let (details, app, (user_id, token)) = create_app_with_fixtures(move |det| {
        let email = email_clone.clone();
        let password = password_clone.clone();
        async move {
            let password_hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST).unwrap();
            let user_id = Uuid::new_v4();
            let now = Utc::now();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO users (id, nickname, name, password_hash, created_at) VALUES (?, ?, ?, ?, ?)",
                    rusqlite::params![&user_id, "testnick", "Test User", &password_hash, now],
                )
                .unwrap();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO emails (email, user_id, is_verified) VALUES (?, ?, 1)",
                    rusqlite::params![&email, &user_id],
                )
                .unwrap();

            let token = "b".repeat(128);
            let session_id = Uuid::new_v4();
            let expires_at = now + Duration::days(30);
            det.sqlite
                .execute_with_params(
                    "INSERT INTO sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
                    rusqlite::params![&session_id, &user_id, &token, &expires_at, &now],
                )
                .unwrap();

            (user_id, token)
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
            token,
        },
    ))
}

// --- GET /api/auth/me ---

#[actix_rt::test]
async fn test_me_returns_profile() {
    let (_details, app, user) = create_app_with_user_with_profile().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["name"], "Test User");
    assert_eq!(body["nickname"], "testnick");
    assert!(body["emails"].as_array().is_some());
    assert_eq!(body["emails"][0]["email"], user.email);
    assert_eq!(body["emails"][0]["is_verified"], true);
}

#[actix_rt::test]
async fn test_me_returns_null_name_nickname_when_not_set() {
    let (_details, app, user) = create_app_with_logged_in_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert!(body["name"].is_null());
    assert!(body["nickname"].is_null());
    assert_eq!(body["emails"][0]["email"], user.email);
}

#[actix_rt::test]
async fn test_me_returns_user_id() {
    let (_details, app, user) = create_app_with_logged_in_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;

    let returned_id: Uuid = serde_json::from_value(body["id"].clone()).unwrap();
    assert_eq!(returned_id, user.user_id);
}

#[actix_rt::test]
async fn test_me_without_auth_returns_401() {
    let (_details, app, _user) = create_app_with_logged_in_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/me")
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_me_with_invalid_token_returns_401() {
    let (_details, app, _user) = create_app_with_logged_in_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/me")
        .insert_header(("Authorization", "Bearer invalid_token_here"))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_me_with_expired_session_returns_401() {
    let test_email = unique_email();
    let email_clone = test_email.clone();

    let (_details, app, token) = create_app_with_fixtures(move |det| {
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

            let token = "c".repeat(128);
            let session_id = Uuid::new_v4();
            let expired_at = now - Duration::minutes(1);
            det.sqlite
                .execute_with_params(
                    "INSERT INTO sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
                    rusqlite::params![&session_id, &user_id, &token, &expired_at, &now],
                )
                .unwrap();

            token
        }
    })
    .await
    .unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

// --- POST /api/auth/change-profile ---

#[actix_rt::test]
async fn test_update_profile_sets_name_and_nickname() {
    let (_details, app, user) = create_app_with_logged_in_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/change-profile")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({"name": "Alice", "nickname": "alice42"}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["name"], "Alice");
    assert_eq!(body["nickname"], "alice42");
    assert_eq!(body["emails"][0]["email"], user.email);
}

#[actix_rt::test]
async fn test_update_profile_persists_changes() {
    let (_details, app, user) = create_app_with_logged_in_user().await.unwrap();

    // Update
    let req = TestRequest::post()
        .uri("/api/auth/change-profile")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({"name": "Bob", "nickname": "bob99"}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let _body: Value = actix_web::test::read_body_json(resp).await;

    // Verify via /me
    let req = TestRequest::post()
        .uri("/api/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .to_request();
    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;

    assert_eq!(body["name"], "Bob");
    assert_eq!(body["nickname"], "bob99");
}

#[actix_rt::test]
async fn test_update_profile_clears_fields_with_null() {
    let (_details, app, user) = create_app_with_user_with_profile().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/change-profile")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({"name": null, "nickname": null}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert!(body["name"].is_null());
    assert!(body["nickname"].is_null());
}

#[actix_rt::test]
async fn test_update_profile_without_auth_returns_401() {
    let (_details, app, _user) = create_app_with_logged_in_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/change-profile")
        .set_json(json!({"name": "Alice"}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_update_profile_partial_update_name_only() {
    let (_details, app, user) = create_app_with_user_with_profile().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/change-profile")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({"name": "New Name", "nickname": "testnick"}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["name"], "New Name");
    assert_eq!(body["nickname"], "testnick");
}

// --- POST /api/auth/change-password ---

#[actix_rt::test]
async fn test_change_password_successful() {
    let (_details, app, user) = create_app_with_logged_in_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/change-password")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({
            "old_password": user.password,
            "new_password": "NewSecurePass456",
            "new_password_confirm": "NewSecurePass456"
        }))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["message"], "Password changed successfully");
}

#[actix_rt::test]
async fn test_change_password_login_works_with_new_password() {
    let (_details, app, user) = create_app_with_logged_in_user().await.unwrap();
    let new_password = "BrandNewPass789";

    // Change password
    let req = TestRequest::post()
        .uri("/api/auth/change-password")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({
            "old_password": user.password,
            "new_password": new_password,
            "new_password_confirm": new_password
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
}

#[actix_rt::test]
async fn test_change_password_old_password_stops_working() {
    let (_details, app, user) = create_app_with_logged_in_user().await.unwrap();
    let new_password = "BrandNewPass789";

    // Change password
    let req = TestRequest::post()
        .uri("/api/auth/change-password")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({
            "old_password": user.password,
            "new_password": new_password,
            "new_password_confirm": new_password
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let _body: Value = actix_web::test::read_body_json(resp).await;

    // Login with old password should fail
    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": user.email, "password": user.password}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_change_password_wrong_old_password() {
    let (_details, app, user) = create_app_with_logged_in_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/change-password")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({
            "old_password": "WrongOldPassword",
            "new_password": "NewPass123",
            "new_password_confirm": "NewPass123"
        }))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["code"], "InvalidCredentials");
}

#[actix_rt::test]
async fn test_change_password_mismatch() {
    let (_details, app, user) = create_app_with_logged_in_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/change-password")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({
            "old_password": user.password,
            "new_password": "NewPass123",
            "new_password_confirm": "DifferentPass456"
        }))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 400);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["code"], "PasswordMismatch");
}

#[actix_rt::test]
async fn test_change_password_without_auth_returns_401() {
    let (_details, app, user) = create_app_with_logged_in_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/change-password")
        .set_json(json!({
            "old_password": user.password,
            "new_password": "NewPass123",
            "new_password_confirm": "NewPass123"
        }))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_change_password_with_invalid_token_returns_401() {
    let (_details, app, user) = create_app_with_logged_in_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/change-password")
        .insert_header(("Authorization", "Bearer totally_invalid_token"))
        .set_json(json!({
            "old_password": user.password,
            "new_password": "NewPass123",
            "new_password_confirm": "NewPass123"
        }))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

// --- Auth middleware edge cases ---

#[actix_rt::test]
async fn test_malformed_auth_header_returns_401() {
    let (_details, app, _user) = create_app_with_logged_in_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/me")
        .insert_header(("Authorization", "NotBearer sometoken"))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

// --- Full flow: login then use token ---

#[actix_rt::test]
async fn test_login_then_me_full_flow() {
    let test_email = unique_email();
    let test_password = "SecurePass123".to_string();
    let email_clone = test_email.clone();
    let password_clone = test_password.clone();

    let (_details, app, _) = create_app_with_fixtures(move |det| {
        let email = email_clone.clone();
        let password = password_clone.clone();
        async move {
            let password_hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST).unwrap();
            let user_id = Uuid::new_v4();
            let now = Utc::now();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO users (id, nickname, name, password_hash, created_at) VALUES (?, ?, ?, ?, ?)",
                    rusqlite::params![&user_id, "mynick", "My Name", &password_hash, now],
                )
                .unwrap();
            det.sqlite
                .execute_with_params(
                    "INSERT INTO emails (email, user_id, is_verified) VALUES (?, ?, 1)",
                    rusqlite::params![&email, &user_id],
                )
                .unwrap();
        }
    })
    .await
    .unwrap();

    // Login
    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": test_email, "password": test_password}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = actix_web::test::read_body_json(resp).await;
    let token = body["token"].as_str().unwrap();

    // Get profile
    let req = TestRequest::post()
        .uri("/api/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["name"], "My Name");
    assert_eq!(body["nickname"], "mynick");
    assert_eq!(body["emails"][0]["email"], test_email);
}

#[actix_rt::test]
async fn test_login_then_change_password_then_login_again() {
    let test_email = unique_email();
    let test_password = "SecurePass123".to_string();
    let new_password = "BrandNewPass456";
    let email_clone = test_email.clone();
    let password_clone = test_password.clone();

    let (_details, app, _) = create_app_with_fixtures(move |det| {
        let email = email_clone.clone();
        let password = password_clone.clone();
        async move {
            let password_hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST).unwrap();
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
        }
    })
    .await
    .unwrap();

    // Login
    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": test_email, "password": test_password}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = actix_web::test::read_body_json(resp).await;
    let token = body["token"].as_str().unwrap();

    // Change password
    let req = TestRequest::post()
        .uri("/api/auth/change-password")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({
            "old_password": test_password,
            "new_password": new_password,
            "new_password_confirm": new_password
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let _body: Value = actix_web::test::read_body_json(resp).await;

    // Login with new password
    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": test_email, "password": new_password}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Login with old password fails
    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": test_email, "password": test_password}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}
