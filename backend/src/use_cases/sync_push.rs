use crate::models::chat::{ChatId, MessageId};
use crate::providers::chat_storage::ChatStorageError;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;
use crate::providers::personalized_file_storage::PersonalizedFileStorage;
use crate::providers::sync_ledger::SyncLedgerStorage;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Chat storage error: {0}")]
    Storage(#[from] ChatStorageError),
    #[error("Invalid entity ID: {0}")]
    InvalidId(String),
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::Storage(ChatStorageError::VersionConflict { expected, actual }) => {
                crate::api::Error {
                    code: "VersionConflict".to_string(),
                    message: format!(
                        "Version conflict: expected {}, server has {}",
                        expected, actual
                    ),
                }
            }
            other => crate::api::Error {
                code: "InternalServerError".to_string(),
                message: other.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChangeEntry {
    pub entity_type: String,
    pub entity_id: String,
    pub chat_id: String,
    pub data: serde_json::Value,
    pub action: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Input {
    pub expected_version: u64,
    pub changes: Vec<ChangeEntry>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub new_version: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConflictOutput {
    pub server_version: u64,
    pub conflicting_entities: Vec<String>,
}

pub async fn command(
    file_storage: &PersonalizedFileStorage,
    chat_storage: &PersonalizedChatStorage,
    input: Input,
) -> Result<Output, Error> {
    let ledger = SyncLedgerStorage::new(file_storage.clone());
    let current_ledger = ledger.load().await?;

    // Check version
    if current_ledger.global_version != input.expected_version {
        return Err(Error::Storage(ChatStorageError::VersionConflict {
            expected: input.expected_version,
            actual: current_ledger.global_version,
        }));
    }

    let mut last_version = current_ledger.global_version;

    for change in &input.changes {
        let chat_id: ChatId = Uuid::parse_str(&change.chat_id)
            .map_err(|_| Error::InvalidId(change.chat_id.clone()))?;
        let cs = chat_storage.get_chat_storage(chat_id);

        match change.entity_type.as_str() {
            "Draft" => {
                let entity_id: MessageId = Uuid::parse_str(&change.entity_id)
                    .map_err(|_| Error::InvalidId(change.entity_id.clone()))?;

                match change.action.as_str() {
                    "Created" | "Updated" => {
                        let content = change.data["content"]
                            .as_str()
                            .unwrap_or_default();
                        let model = change.data["model"]
                            .as_str()
                            .unwrap_or_default();
                        cs.get_or_create_chat(model).await?;
                        last_version = cs.save_draft(entity_id, content).await?;
                    }
                    "Deleted" => {
                        // Draft deletion is handled by send_message
                    }
                    _ => {}
                }
            }
            _ => {
                // Messages and chats are server-authoritative, clients don't push them
            }
        }
    }

    Ok(Output {
        new_version: last_version,
    })
}
