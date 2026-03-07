use uuid::Uuid;

pub fn random_bucket_prefix() -> String {
    format!("test-{}-", Uuid::new_v4().simple())
}

pub fn temp_sqlite_path() -> String {
    format!("/tmp/test-{}.db", Uuid::new_v4().simple())
}

pub fn unique_email() -> String {
    format!("test+{}@example.com", Uuid::new_v4().simple())
}
