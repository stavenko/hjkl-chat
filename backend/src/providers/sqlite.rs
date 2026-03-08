use crate::providers::local_filesystem::LocalFileSystemProvider;
use crate::providers::s3::S3Provider;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(thiserror::Error, Debug)]
pub enum SQLiteProviderError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("S3 error: {0}")]
    S3(#[from] crate::providers::s3::S3ProviderError),
    #[error("Local FS error: {0}")]
    LocalFs(#[from] crate::providers::local_filesystem::LocalFileSystemProviderError),
    #[error("Lock error: {0}")]
    Lock(String),
}

pub type SQLiteProviderResult<T> = Result<T, SQLiteProviderError>;

pub struct SQLiteProvider {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
    s3_provider: Arc<S3Provider>,
    fs_provider: Arc<LocalFileSystemProvider>,
    s3_object_path: String,
}

impl Clone for SQLiteProvider {
    fn clone(&self) -> Self {
        SQLiteProvider {
            conn: Arc::clone(&self.conn),
            db_path: self.db_path.clone(),
            s3_provider: Arc::clone(&self.s3_provider),
            fs_provider: Arc::clone(&self.fs_provider),
            s3_object_path: self.s3_object_path.clone(),
        }
    }
}

impl SQLiteProvider {
    pub async fn new(
        s3_provider: Arc<S3Provider>,
        fs_provider: Arc<LocalFileSystemProvider>,
        s3_object_path: &str,
    ) -> SQLiteProviderResult<Self> {
        let db_path = if s3_provider.object_exists(s3_object_path).await? {
            let data = s3_provider.get_object(s3_object_path).await?;
            fs_provider.save(data)?
        } else {
            let path = fs_provider.get_path(s3_object_path)?;
            let conn = Connection::open(&path)?;
            run_migrations(&conn)?;
            path
        };

        let conn = Connection::open(&db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;

        Ok(SQLiteProvider {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
            s3_provider,
            fs_provider,
            s3_object_path: s3_object_path.to_string(),
        })
    }

    pub fn get_connection(&self) -> SQLiteProviderResult<std::sync::MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|e| SQLiteProviderError::Lock(e.to_string()))
    }

    #[allow(dead_code)]
    pub fn execute(&self, sql: &str, params: &[rusqlite::types::ValueRef<'_>]) -> SQLiteProviderResult<usize> {
        let conn = self.get_connection()?;
        let value_params: Vec<rusqlite::types::Value> = params.iter().map(|v| rusqlite::types::Value::from(*v)).collect();
        let result = conn.execute(sql, rusqlite::params_from_iter(value_params))?;
        self.dump_to_s3();
        Ok(result)
    }

    pub fn execute_with_params<P>(&self, sql: &str, params: P) -> SQLiteProviderResult<usize>
    where
        P: rusqlite::Params,
    {
        let conn = self.get_connection()?;
        let result = conn.execute(sql, params)?;
        self.dump_to_s3();
        Ok(result)
    }

    pub fn query_one<T, F>(&self, sql: &str, params: &[rusqlite::types::ValueRef<'_>], mut f: F) -> SQLiteProviderResult<Option<T>>
    where
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
    {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(sql)?;
        let value_params: Vec<rusqlite::types::Value> = params.iter().map(|v| rusqlite::types::Value::from(*v)).collect();
        let mut rows = stmt.query(rusqlite::params_from_iter(value_params))?;
        if let Some(row) = rows.next()? {
            Ok(Some(f(row)?))
        } else {
            Ok(None)
        }
    }

    #[allow(dead_code)]
    pub fn query_all<T, F>(&self, sql: &str, params: &[rusqlite::types::ValueRef<'_>], mut f: F) -> SQLiteProviderResult<Vec<T>>
    where
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
    {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(sql)?;
        let value_params: Vec<rusqlite::types::Value> = params.iter().map(|v| rusqlite::types::Value::from(*v)).collect();
        let mut rows = stmt.query(rusqlite::params_from_iter(value_params))?;
        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            results.push(f(row)?);
        }
        Ok(results)
    }

    pub fn dump_to_s3(&self) {
        let db_path = self.db_path.clone();
        let s3_provider = self.s3_provider.clone();
        let fs_provider = self.fs_provider.clone();
        let s3_object_path = self.s3_object_path.clone();

        tokio::spawn(async move {
            let data = match fs_provider.read(&db_path) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Failed to read database file: {}", e);
                    return;
                }
            };

            if let Err(e) = s3_provider.put_object(&s3_object_path, &data).await {
                eprintln!("Failed to upload database to S3: {}", e);
            }
        });
    }

    #[cfg(test)]
    pub fn new_for_test(
        conn: Connection,
        db_path: PathBuf,
        s3_provider: Arc<S3Provider>,
        fs_provider: Arc<LocalFileSystemProvider>,
        s3_object_path: String,
    ) -> Self {
        SQLiteProvider {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
            s3_provider,
            fs_provider,
            s3_object_path,
        }
    }
}

fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id UUID PRIMARY KEY,
            user_id UUID REFERENCES users(id),
            access_token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS registration_sessions (
            id UUID PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            verification_code TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            resend_available_at TIMESTAMP NOT NULL
        )",
        [],
    )?;

    Ok(())
}