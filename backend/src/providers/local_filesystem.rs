use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum LocalFileSystemProviderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path must be relative to root")]
    InvalidPath,
}

pub type LocalFileSystemProviderResult<T> = Result<T, LocalFileSystemProviderError>;

pub struct LocalFileSystemProvider {
    root: PathBuf,
}

impl LocalFileSystemProvider {
    pub fn new(root: PathBuf) -> LocalFileSystemProviderResult<Self> {
        fs::create_dir_all(&root)?;
        let test_file = root.join(".initialized");
        fs::write(&test_file, "")?;
        fs::remove_file(test_file)?;

        Ok(LocalFileSystemProvider { root })
    }

    pub fn save(&self, object: Vec<u8>) -> LocalFileSystemProviderResult<PathBuf> {
        let filename = format!("{}.dat", Uuid::new_v4());
        let path = self.root.join(&filename);
        fs::write(&path, object)?;
        Ok(path)
    }

    #[allow(dead_code)]
    pub fn delete(&self, path: &Path) -> LocalFileSystemProviderResult<()> {
        if !path.starts_with(&self.root) {
            return Err(LocalFileSystemProviderError::InvalidPath);
        }
        fs::remove_file(path)?;
        Ok(())
    }

    pub fn read(&self, path: &Path) -> LocalFileSystemProviderResult<Vec<u8>> {
        if !path.starts_with(&self.root) {
            return Err(LocalFileSystemProviderError::InvalidPath);
        }
        Ok(fs::read(path)?)
    }

    pub fn get_path(&self, filename: &str) -> LocalFileSystemProviderResult<PathBuf> {
        let path = self.root.join(filename);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(path)
    }
}
