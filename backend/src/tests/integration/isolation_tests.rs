use std::collections::HashSet;

use crate::tests::utils::{random_bucket_prefix, temp_sqlite_path, unique_email};

#[actix_rt::test]
async fn test_random_bucket_prefix_format() {
    let prefix = random_bucket_prefix();

    assert!(
        prefix.starts_with("test-"),
        "Bucket prefix should start with 'test-'"
    );
    assert!(
        prefix.ends_with("-"),
        "Bucket prefix should end with '-'"
    );
    assert!(
        prefix.len() > 7,
        "Bucket prefix should have content between 'test-' and final '-'"
    );
}

#[actix_rt::test]
async fn test_random_bucket_prefix_uniqueness() {
    let mut prefixes: HashSet<String> = HashSet::new();

    for _ in 0..100 {
        let prefix = random_bucket_prefix();
        assert!(
            prefixes.insert(prefix),
            "Bucket prefix should be unique"
        );
    }
}

#[actix_rt::test]
async fn test_temp_sqlite_path_format() {
    let path = temp_sqlite_path();

    assert!(
        path.starts_with("/tmp/test-"),
        "Temp SQLite path should start with '/tmp/test-'"
    );
    assert!(
        path.ends_with(".db"),
        "Temp SQLite path should end with '.db'"
    );
    assert!(
        path.len() > 12,
        "Temp SQLite path should have content between '/tmp/test-' and '.db'"
    );
}

#[actix_rt::test]
async fn test_temp_sqlite_path_uniqueness() {
    let mut paths: HashSet<String> = HashSet::new();

    for _ in 0..100 {
        let path = temp_sqlite_path();
        assert!(paths.insert(path), "Temp SQLite path should be unique");
    }
}

#[actix_rt::test]
async fn test_unique_email_format() {
    let email = unique_email();

    assert!(
        email.starts_with("test+"),
        "Email should start with 'test+'"
    );
    assert!(
        email.ends_with("@example.com"),
        "Email should end with '@example.com'"
    );
    assert!(
        email.len() > 18,
        "Email should have content between 'test+' and '@example.com'"
    );
}

#[actix_rt::test]
async fn test_unique_email_uniqueness() {
    let mut emails: HashSet<String> = HashSet::new();

    for _ in 0..100 {
        let email = unique_email();
        assert!(emails.insert(email), "Email should be unique");
    }
}

#[actix_rt::test]
async fn test_isolation_utils_combined_uniqueness() {
    let mut prefixes: HashSet<String> = HashSet::new();
    let mut paths: HashSet<String> = HashSet::new();
    let mut emails: HashSet<String> = HashSet::new();

    for _ in 0..50 {
        prefixes.insert(random_bucket_prefix());
        paths.insert(temp_sqlite_path());
        emails.insert(unique_email());
    }

    assert_eq!(prefixes.len(), 50, "All bucket prefixes should be unique");
    assert_eq!(paths.len(), 50, "All temp paths should be unique");
    assert_eq!(emails.len(), 50, "All emails should be unique");
}