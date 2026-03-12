use actix_web::dev::Service;
use actix_web::test::TestRequest;
use bcrypt::hash;
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::tests::test_app::{create_app_with_fixtures, AppDetails};
use crate::tests::utils::unique_email;

struct AuthenticatedUser {
    token: String,
    user_id: Uuid,
}

async fn create_app_with_authenticated_user() -> anyhow::Result<(
    AppDetails,
    impl Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
    >,
    AuthenticatedUser,
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

    // Login to get a token
    let req = TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({"email": test_email, "password": test_password}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    let body: Value = actix_web::test::read_body_json(resp).await;
    let token = body["token"].as_str().unwrap().to_string();

    Ok((details, app, AuthenticatedUser { token, user_id }))
}

/// Helper: push a draft via sync/push and return the new version
async fn push_draft(
    app: &impl Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
    >,
    token: &str,
    chat_id: &Uuid,
    message_id: &Uuid,
    content: &str,
    model: &str,
    expected_version: u64,
) -> u64 {
    let req = TestRequest::post()
        .uri("/api/sync/push")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(json!({
            "expected_version": expected_version,
            "changes": [{
                "entity_type": "Draft",
                "entity_id": message_id.to_string(),
                "chat_id": chat_id.to_string(),
                "data": {"content": content, "model": model},
                "action": "Created"
            }]
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    body["new_version"].as_u64().unwrap()
}

// --- Sync Pull Tests ---

#[actix_rt::test]
async fn test_sync_pull_empty_returns_version_zero() {
    let (_details, app, user) = create_app_with_authenticated_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/sync/pull")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({"since_version": 0}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["current_version"], 0);
    assert_eq!(body["entries"].as_array().unwrap().len(), 0);
    assert_eq!(body["data"]["messages"].as_array().unwrap().len(), 0);
    assert_eq!(body["data"]["chats"].as_array().unwrap().len(), 0);
    assert_eq!(body["data"]["drafts"].as_array().unwrap().len(), 0);
}

#[actix_rt::test]
async fn test_sync_pull_returns_changes_after_push() {
    let (_details, app, user) = create_app_with_authenticated_user().await.unwrap();

    let chat_id = Uuid::new_v4();
    let message_id = Uuid::new_v4();

    // Push a draft
    let version = push_draft(&app, &user.token, &chat_id, &message_id, "Hello world", "test-model", 0).await;
    assert!(version > 0);

    // Now pull from version 0 — should get changes
    let req = TestRequest::post()
        .uri("/api/sync/pull")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({"since_version": 0}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert!(body["current_version"].as_u64().unwrap() > 0);
    assert!(!body["entries"].as_array().unwrap().is_empty());

    // Should have a chat in data (created by push)
    assert!(!body["data"]["chats"].as_array().unwrap().is_empty());
}

#[actix_rt::test]
async fn test_sync_pull_since_current_returns_no_changes() {
    let (_details, app, user) = create_app_with_authenticated_user().await.unwrap();

    let chat_id = Uuid::new_v4();
    let message_id = Uuid::new_v4();

    // Push a draft
    let current_version = push_draft(&app, &user.token, &chat_id, &message_id, "Hello world", "test-model", 0).await;

    // Pull since current version — should get nothing new
    let req = TestRequest::post()
        .uri("/api/sync/pull")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({"since_version": current_version}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["entries"].as_array().unwrap().len(), 0);
    assert_eq!(body["data"]["messages"].as_array().unwrap().len(), 0);
}

// --- Sync Push Tests ---

#[actix_rt::test]
async fn test_sync_push_draft_succeeds() {
    let (_details, app, user) = create_app_with_authenticated_user().await.unwrap();

    let chat_id = Uuid::new_v4();
    let message_id = Uuid::new_v4();

    let req = TestRequest::post()
        .uri("/api/sync/push")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({
            "expected_version": 0,
            "changes": [{
                "entity_type": "Draft",
                "entity_id": message_id.to_string(),
                "chat_id": chat_id.to_string(),
                "data": {"content": "Pushed draft", "model": "test-model"},
                "action": "Created"
            }]
        }))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert!(body["new_version"].as_u64().unwrap() > 0);
}

#[actix_rt::test]
async fn test_sync_push_version_conflict_returns_409() {
    let (_details, app, user) = create_app_with_authenticated_user().await.unwrap();

    let chat_id = Uuid::new_v4();
    let message_id = Uuid::new_v4();

    // First push succeeds
    let req = TestRequest::post()
        .uri("/api/sync/push")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({
            "expected_version": 0,
            "changes": [{
                "entity_type": "Draft",
                "entity_id": message_id.to_string(),
                "chat_id": chat_id.to_string(),
                "data": {"content": "Draft v1", "model": "test-model"},
                "action": "Created"
            }]
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Second push with stale version should fail with 409
    let req = TestRequest::post()
        .uri("/api/sync/push")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({
            "expected_version": 0,
            "changes": [{
                "entity_type": "Draft",
                "entity_id": message_id.to_string(),
                "chat_id": chat_id.to_string(),
                "data": {"content": "Draft v2", "model": "test-model"},
                "action": "Updated"
            }]
        }))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 409);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["code"], "VersionConflict");
}

// --- End-to-End: Push Draft → Send → Pull ---

#[actix_rt::test]
async fn test_full_flow_push_send_and_sync_pull() {
    let (_details, app, user) = create_app_with_authenticated_user().await.unwrap();

    let chat_id = Uuid::new_v4();
    let message_id = Uuid::new_v4();

    // 1. Push draft via sync
    push_draft(&app, &user.token, &chat_id, &message_id, "Hello assistant", "test-model", 0).await;

    // 2. Send message
    let req = TestRequest::post()
        .uri(&format!("/api/chat/{}/send-message", chat_id))
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({
            "message_id": message_id,
            "model": "test-model"
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let send_body: Value = actix_web::test::read_body_json(resp).await;
    assert!(send_body["assistant_message_id"].is_string());

    // Wait for the async assistant message processing
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 3. Sync pull — should see user message, chat, and possibly assistant message
    let req = TestRequest::post()
        .uri("/api/sync/pull")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({"since_version": 0}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert!(body["current_version"].as_u64().unwrap() > 0);

    // Should have entries for chat creation, draft, message, etc.
    let entries = body["entries"].as_array().unwrap();
    assert!(entries.len() >= 2, "Should have multiple sync entries, got {}", entries.len());

    // Should have the chat in data
    let chats = body["data"]["chats"].as_array().unwrap();
    assert_eq!(chats.len(), 1);
    assert_eq!(chats[0]["id"], chat_id.to_string());

    // Should have at least the user message
    let messages = body["data"]["messages"].as_array().unwrap();
    assert!(messages.len() >= 1, "Should have at least 1 message, got {}", messages.len());
}

// --- Version Incrementing ---

#[actix_rt::test]
async fn test_versions_increment_monotonically() {
    let (_details, app, user) = create_app_with_authenticated_user().await.unwrap();

    let chat_id = Uuid::new_v4();

    let mut versions: Vec<u64> = Vec::new();
    let mut current_version = 0u64;

    // Push multiple drafts and track versions
    for i in 0..5 {
        let message_id = Uuid::new_v4();
        let version = push_draft(
            &app, &user.token, &chat_id, &message_id,
            &format!("Draft {}", i), "test-model", current_version,
        ).await;
        versions.push(version);
        current_version = version;
    }

    // Each version should be strictly greater than the previous
    for window in versions.windows(2) {
        assert!(
            window[1] > window[0],
            "Versions should be strictly increasing: {} should be > {}",
            window[1],
            window[0]
        );
    }
}

// --- Unauthenticated Access ---

#[actix_rt::test]
async fn test_sync_pull_without_auth_returns_401() {
    let (_details, app, _user) = create_app_with_authenticated_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/sync/pull")
        .set_json(json!({"since_version": 0}))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_sync_push_without_auth_returns_401() {
    let (_details, app, _user) = create_app_with_authenticated_user().await.unwrap();

    let req = TestRequest::post()
        .uri("/api/sync/push")
        .set_json(json!({
            "expected_version": 0,
            "changes": []
        }))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

// --- Sync Push then Pull round-trip ---

#[actix_rt::test]
async fn test_push_then_pull_returns_pushed_data() {
    let (_details, app, user) = create_app_with_authenticated_user().await.unwrap();

    let chat_id = Uuid::new_v4();
    let message_id = Uuid::new_v4();

    // Push a draft
    let req = TestRequest::post()
        .uri("/api/sync/push")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({
            "expected_version": 0,
            "changes": [{
                "entity_type": "Draft",
                "entity_id": message_id.to_string(),
                "chat_id": chat_id.to_string(),
                "data": {"content": "Synced draft content", "model": "test-model"},
                "action": "Created"
            }]
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let push_body: Value = actix_web::test::read_body_json(resp).await;
    let new_version = push_body["new_version"].as_u64().unwrap();
    assert!(new_version > 0);

    // Pull from version 0 — should see the draft
    let req = TestRequest::post()
        .uri("/api/sync/pull")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({"since_version": 0}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    assert_eq!(body["current_version"].as_u64().unwrap(), new_version);

    // Should have a draft in the data
    let drafts = body["data"]["drafts"].as_array().unwrap();
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0]["content"], "Synced draft content");
}

// --- Multiple syncs with incremental pull ---

#[actix_rt::test]
async fn test_incremental_pull_only_returns_new_changes() {
    let (_details, app, user) = create_app_with_authenticated_user().await.unwrap();

    let chat_id = Uuid::new_v4();

    // Push first draft
    let msg1 = Uuid::new_v4();
    let version_after_first = push_draft(&app, &user.token, &chat_id, &msg1, "First draft", "test-model", 0).await;

    // Push second draft
    let msg2 = Uuid::new_v4();
    push_draft(&app, &user.token, &chat_id, &msg2, "Second draft", "test-model", version_after_first).await;

    // Pull since version_after_first — should only get second draft's entries
    let req = TestRequest::post()
        .uri("/api/sync/pull")
        .insert_header(("Authorization", format!("Bearer {}", user.token)))
        .set_json(json!({"since_version": version_after_first}))
        .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = actix_web::test::read_body_json(resp).await;
    let entries = body["entries"].as_array().unwrap();

    // All entries should have version > version_after_first
    for entry in entries {
        assert!(
            entry["version"].as_u64().unwrap() > version_after_first,
            "Entry version {} should be > {}",
            entry["version"],
            version_after_first
        );
    }
}
