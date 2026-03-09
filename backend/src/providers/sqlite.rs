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
            fs_provider.get_path(s3_object_path)?
        };

        let conn = Connection::open(&db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        create_schema(&conn)?;

        Ok(SQLiteProvider {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
            s3_provider,
            fs_provider,
            s3_object_path: s3_object_path.to_string(),
        })
    }

    pub fn get_connection(&self) -> SQLiteProviderResult<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|e| SQLiteProviderError::Lock(e.to_string()))
    }

    #[allow(dead_code)]
    pub fn execute(
        &self,
        sql: &str,
        params: &[rusqlite::types::ValueRef<'_>],
    ) -> SQLiteProviderResult<usize> {
        let result = {
            let conn = self.get_connection()?;
            let value_params: Vec<rusqlite::types::Value> = params
                .iter()
                .map(|v| rusqlite::types::Value::from(*v))
                .collect();
            conn.execute(sql, rusqlite::params_from_iter(value_params))?
        };
        self.dump_to_s3();
        Ok(result)
    }

    pub fn execute_with_params<P>(&self, sql: &str, params: P) -> SQLiteProviderResult<usize>
    where
        P: rusqlite::Params,
    {
        let result = {
            let conn = self.get_connection()?;
            conn.execute(sql, params)?
        };
        self.dump_to_s3();
        Ok(result)
    }

    pub fn query_one<T, F>(
        &self,
        sql: &str,
        params: &[rusqlite::types::ValueRef<'_>],
        mut f: F,
    ) -> SQLiteProviderResult<Option<T>>
    where
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
    {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(sql)?;
        let value_params: Vec<rusqlite::types::Value> = params
            .iter()
            .map(|v| rusqlite::types::Value::from(*v))
            .collect();
        let mut rows = stmt.query(rusqlite::params_from_iter(value_params))?;
        if let Some(row) = rows.next()? {
            Ok(Some(f(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn query_one_with_params<T, F, P>(
        &self,
        sql: &str,
        params: P,
        mut f: F,
    ) -> SQLiteProviderResult<Option<T>>
    where
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
        P: rusqlite::Params,
    {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query(params)?;
        if let Some(row) = rows.next()? {
            Ok(Some(f(row)?))
        } else {
            Ok(None)
        }
    }

    #[allow(dead_code)]
    pub fn query_all<T, F>(
        &self,
        sql: &str,
        params: &[rusqlite::types::ValueRef<'_>],
        mut f: F,
    ) -> SQLiteProviderResult<Vec<T>>
    where
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
    {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(sql)?;
        let value_params: Vec<rusqlite::types::Value> = params
            .iter()
            .map(|v| rusqlite::types::Value::from(*v))
            .collect();
        let mut rows = stmt.query(rusqlite::params_from_iter(value_params))?;
        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            results.push(f(row)?);
        }
        Ok(results)
    }

    pub fn migrate(&self) -> SQLiteProviderResult<()> {
        let conn = self.get_connection()?;
        run_migrations(&conn)?;
        drop(conn);
        self.dump_to_s3();
        Ok(())
    }

    pub fn dump_to_s3(&self) {
        if let Ok(conn) = self.get_connection() {
            if let Err(e) = conn.pragma_update(None, "wal_checkpoint", "TRUNCATE") {
                eprintln!("Failed to checkpoint WAL: {}", e);
            }
        }

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

}

fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            nickname TEXT,
            name TEXT,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS emails (
            email TEXT PRIMARY KEY,
            user_id UUID NOT NULL REFERENCES users(id),
            is_verified BOOLEAN NOT NULL DEFAULT 0
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id UUID PRIMARY KEY,
            user_id UUID REFERENCES users(id),
            token TEXT NOT NULL,
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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS password_restore_sessions (
            id UUID PRIMARY KEY,
            user_id UUID REFERENCES users(id),
            email TEXT NOT NULL,
            verification_code TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            resend_available_at TIMESTAMP NOT NULL
        )",
        [],
    )?;

    Ok(())
}

fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    let users_table_exists = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='users'")?
        .exists([])?;

    if !users_table_exists {
        return Ok(());
    }

    let mut stmt = conn.prepare("PRAGMA table_info(users)")?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if columns.contains(&"email".to_string()) {
        conn.pragma_update(None, "legacy_alter_table", "ON")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS emails (
                email TEXT PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id),
                is_verified BOOLEAN NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO emails (email, user_id, is_verified)
             SELECT email, id, 1 FROM users",
            [],
        )?;

        conn.execute("ALTER TABLE users RENAME TO users_old", [])?;
        conn.execute(
            "CREATE TABLE users (
                id UUID PRIMARY KEY,
                nickname TEXT,
                name TEXT,
                password_hash TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "INSERT INTO users (id, password_hash, created_at)
             SELECT id, password_hash, created_at FROM users_old",
            [],
        )?;
        conn.execute("DROP TABLE users_old", [])?;

        conn.pragma_update(None, "legacy_alter_table", "OFF")?;

        println!("Migration complete: moved emails out of users table");
    }

    Ok(())
}

