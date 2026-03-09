use actix_web::test::TestRequest;
use actix_web::dev::Service;
use bcrypt::hash;
use chrono::Utc;
use crate::tests::test_app::create_app_with_fixtures;
use serde_json::json;
use uuid::Uuid;

async fn create_app_with_test_user() -> anyhow::Result<(
    crate::tests::test_app::AppDetails,
    impl Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
    >,
    String,
    String,
)> {
    let test_email = format!("test+{}@example.com", Uuid::new_v4().simple());
    let test_password = "SecurePass123".to_string();
    let email_clone = test_email.clone();
    let password_clone = test_password.clone();

    let (details, app, _) = create_app_with_fixtures(move |det| {
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
        }
    })
    .await?;

    Ok((details, app, test_email, test_password))
}

#[actix_rt::test]
async fn test_login_successful() {
    let (_details, app, email, password) = create_app_with_test_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": email, "password": password}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["user"]["email"], email);
    assert!(body["token"].as_str().map(|s| !s.is_empty()).unwrap_or(false));
}

#[actix_rt::test]
async fn test_login_wrong_password() {
    let (_details, app, email, _password) = create_app_with_test_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": email, "password": "wrong_password"}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_login_nonexistent_email() {
    let (_details, app, _email, _password) = create_app_with_test_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": "nonexistent@example.com", "password": "SecurePass123"}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_login_missing_fields_returns_error() {
    let (_details, app, _email, _password) = create_app_with_test_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": "", "password": "password"}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 400);
}
