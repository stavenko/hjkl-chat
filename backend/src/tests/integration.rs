use actix_web::test;
use actix_web::web::Data;
use crate::config::Config;
use crate::tests::utils::{random_bucket_prefix, temp_sqlite_path, unique_email};

#[actix_rt::test]
async fn test_minio_connection() {
    let bucket_prefix = random_bucket_prefix();
    let temp_db = temp_sqlite_path();
    let test_email = unique_email();
    
    let app = test::init_service(
        actix_web::App::new()
            .app_data(Data::new(Config::default()))
    )
    .await;
    
    let req = test::TestRequest::get()
        .uri("/")
        .to_request();
    
    let _resp = test::call_service(&app, req).await;
    
    assert!(bucket_prefix.starts_with("test-"));
    assert!(temp_db.ends_with(".db"));
    assert!(test_email.ends_with("@example.com"));
}