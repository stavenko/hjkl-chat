use std::env;
use std::path::PathBuf;
use uuid::Uuid;

pub fn generate_random_bucket_prefix() -> String {
    format!(
        "test-{}-",
        Uuid::new_v4().hyphenated().to_string()[..8].to_string()
    )
}

pub fn generate_temp_sqlite_path() -> PathBuf {
    let uuid = Uuid::new_v4().hyphenated().to_string();
    PathBuf::from(env::var("TEMP_DIR").unwrap_or_else(|_| "/tmp".to_string()))
        .join(format!("test-{}.db", uuid))
}

pub fn generate_unique_email() -> String {
    let uuid = Uuid::new_v4().hyphenated().to_string();
    format!("test+{}@example.com", uuid)
}

#[allow(dead_code)]
pub fn generate_unique_email_with_suffix(suffix: &str) -> String {
    let uuid = Uuid::new_v4().hyphenated().to_string();
    format!("test+{}+{}@example.com", suffix, uuid)
}
