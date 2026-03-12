use crate::tests::utils::{random_bucket_prefix, temp_sqlite_path, unique_email};

mod minio_tests;
pub mod mailhog_tests;
mod isolation_tests;
mod concurrent_tests;
mod auth_tests;
mod login_service_tests;
mod profile_service_tests;
mod registration_service_tests;
mod restore_service_tests;
mod sync_tests;

#[actix_rt::test]
async fn test_test_utils_generate_valid_values() {
    let bucket_prefix = random_bucket_prefix();
    let temp_db = temp_sqlite_path();
    let test_email = unique_email();
    
    assert!(bucket_prefix.starts_with("test-"));
    assert!(bucket_prefix.ends_with("-"));
    assert!(temp_db.starts_with("/tmp/test-"));
    assert!(temp_db.ends_with(".db"));
    assert!(test_email.starts_with("test+"));
    assert!(test_email.ends_with("@example.com"));
}

#[actix_rt::test]
async fn test_test_utils_generate_unique_values() {
    let prefixes: Vec<String> = (0..10).map(|_| random_bucket_prefix()).collect();
    let paths: Vec<String> = (0..10).map(|_| temp_sqlite_path()).collect();
    let emails: Vec<String> = (0..10).map(|_| unique_email()).collect();
    
    assert_eq!(prefixes.len(), prefixes.into_iter().collect::<std::collections::HashSet<_>>().len(), "Bucket prefixes should be unique");
    assert_eq!(paths.len(), paths.into_iter().collect::<std::collections::HashSet<_>>().len(), "Temp paths should be unique");
    assert_eq!(emails.len(), emails.into_iter().collect::<std::collections::HashSet<_>>().len(), "Emails should be unique");
}

