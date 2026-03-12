use crate::models::chat::{SyncAction, SyncEntityType, SyncEntry, SyncLedger};
use crate::providers::personalized_file_storage::PersonalizedFileStorage;
use chrono::Utc;

use super::chat_storage::ChatStorageError;

const SYNC_LEDGER_PATH: &str = "sync-ledger.yaml";

pub struct SyncLedgerStorage {
    file_storage: PersonalizedFileStorage,
}

impl SyncLedgerStorage {
    pub fn new(file_storage: PersonalizedFileStorage) -> Self {
        SyncLedgerStorage { file_storage }
    }

    pub async fn load(&self) -> Result<SyncLedger, ChatStorageError> {
        if !self.file_storage.exists(SYNC_LEDGER_PATH).await? {
            return Ok(SyncLedger {
                global_version: 0,
                entries: Vec::new(),
            });
        }
        let data = self.file_storage.get(SYNC_LEDGER_PATH).await?;
        let yaml_str = String::from_utf8(data)?;
        let ledger: SyncLedger = serde_yaml::from_str(&yaml_str)?;
        Ok(ledger)
    }

    pub async fn save(&self, ledger: &SyncLedger) -> Result<(), ChatStorageError> {
        let yaml = serde_yaml::to_string(ledger)?;
        self.file_storage
            .put(SYNC_LEDGER_PATH, yaml.as_bytes())
            .await?;
        Ok(())
    }

    /// Increment version and record a sync entry. Returns the new version.
    pub async fn record(
        &self,
        entity_type: SyncEntityType,
        entity_path: String,
        action: SyncAction,
    ) -> Result<u64, ChatStorageError> {
        let mut ledger = self.load().await?;
        ledger.global_version += 1;
        let version = ledger.global_version;
        ledger.entries.push(SyncEntry {
            version,
            entity_type,
            entity_path,
            action,
            timestamp: Utc::now(),
        });
        self.save(&ledger).await?;
        Ok(version)
    }

    /// Get all entries since a given version
    pub async fn entries_since(
        &self,
        since_version: u64,
    ) -> Result<(u64, Vec<SyncEntry>), ChatStorageError> {
        let ledger = self.load().await?;
        let entries: Vec<SyncEntry> = ledger
            .entries
            .into_iter()
            .filter(|e| e.version > since_version)
            .collect();
        Ok((ledger.global_version, entries))
    }
}
