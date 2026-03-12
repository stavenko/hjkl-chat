use crate::models::chat::{ChatId, ChatMessage, ChatIndex, SyncEntityType};
use crate::providers::chat_storage::ChatStorageError;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;
use crate::providers::personalized_file_storage::PersonalizedFileStorage;
use crate::providers::sync_ledger::SyncLedgerStorage;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Chat storage error: {0}")]
    Storage(#[from] ChatStorageError),
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        crate::api::Error {
            code: "InternalServerError".to_string(),
            message: value.to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Input {
    pub since_version: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncEntryOutput {
    pub version: u64,
    pub entity_type: String,
    pub entity_path: String,
    pub action: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncData {
    pub messages: Vec<ChatMessage>,
    pub chats: Vec<ChatIndex>,
    pub drafts: Vec<ChatMessage>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub current_version: u64,
    pub entries: Vec<SyncEntryOutput>,
    pub data: SyncData,
}

pub async fn command(
    file_storage: &PersonalizedFileStorage,
    chat_storage: &PersonalizedChatStorage,
    input: Input,
) -> Result<Output, Error> {
    let ledger = SyncLedgerStorage::new(file_storage.clone());
    let (current_version, entries) = ledger.entries_since(input.since_version).await?;

    // Collect unique entity references from entries
    let mut chat_ids: HashSet<ChatId> = HashSet::new();
    let mut message_refs: Vec<(ChatId, String)> = Vec::new();
    let mut draft_refs: Vec<(ChatId, String)> = Vec::new();

    for entry in &entries {
        match entry.entity_type {
            SyncEntityType::Chat => {
                if let Some(id) = extract_chat_id(&entry.entity_path) {
                    chat_ids.insert(id);
                }
            }
            SyncEntityType::Message => {
                if let Some((chat_id, msg_id)) = extract_chat_message_id(&entry.entity_path) {
                    chat_ids.insert(chat_id);
                    message_refs.push((chat_id, msg_id));
                }
            }
            SyncEntityType::Draft => {
                if let Some((chat_id, draft_id)) = extract_chat_draft_id(&entry.entity_path) {
                    chat_ids.insert(chat_id);
                    draft_refs.push((chat_id, draft_id));
                }
            }
        }
    }

    // Load referenced data
    let mut chats = Vec::new();
    let mut messages = Vec::new();
    let mut drafts = Vec::new();

    for chat_id in &chat_ids {
        let cs = chat_storage.get_chat_storage(*chat_id);
        match cs.load_index().await {
            Ok(index) => chats.push(index),
            Err(_) => {} // chat might have been deleted
        }
    }

    for (chat_id, msg_id) in &message_refs {
        if let Ok(uuid) = Uuid::parse_str(msg_id) {
            let msg_path = format!("chats/{}/messages/{}.yaml", chat_id, uuid);
            if let Ok(data) = file_storage.get(&msg_path).await {
                if let Ok(yaml_str) = String::from_utf8(data) {
                    if let Ok(msg) = serde_yaml::from_str::<ChatMessage>(&yaml_str) {
                        messages.push(msg);
                    }
                }
            }
        }
    }

    for (chat_id, draft_id) in &draft_refs {
        if let Ok(uuid) = Uuid::parse_str(draft_id) {
            let cs = chat_storage.get_chat_storage(*chat_id);
            match cs.get_draft(uuid).await {
                Ok(draft) => drafts.push(draft),
                Err(_) => {} // draft might have been deleted (sent as message)
            }
        }
    }

    let entry_outputs: Vec<SyncEntryOutput> = entries
        .into_iter()
        .map(|e| SyncEntryOutput {
            version: e.version,
            entity_type: format!("{:?}", e.entity_type),
            entity_path: e.entity_path,
            action: format!("{:?}", e.action),
            timestamp: e.timestamp.to_rfc3339(),
        })
        .collect();

    Ok(Output {
        current_version,
        entries: entry_outputs,
        data: SyncData {
            messages,
            chats,
            drafts,
        },
    })
}

fn extract_chat_id(path: &str) -> Option<ChatId> {
    // "chats/{chat_id}"
    let stripped = path.strip_prefix("chats/")?;
    Uuid::parse_str(stripped).ok()
}

fn extract_chat_message_id(path: &str) -> Option<(ChatId, String)> {
    // "chats/{chat_id}/messages/{msg_id}"
    let stripped = path.strip_prefix("chats/")?;
    let parts: Vec<&str> = stripped.splitn(3, '/').collect();
    if parts.len() >= 3 && parts[1] == "messages" {
        let chat_id = Uuid::parse_str(parts[0]).ok()?;
        Some((chat_id, parts[2].to_string()))
    } else {
        None
    }
}

fn extract_chat_draft_id(path: &str) -> Option<(ChatId, String)> {
    // "chats/{chat_id}/drafts/{draft_id}"
    let stripped = path.strip_prefix("chats/")?;
    let parts: Vec<&str> = stripped.splitn(3, '/').collect();
    if parts.len() >= 3 && parts[1] == "drafts" {
        let chat_id = Uuid::parse_str(parts[0]).ok()?;
        Some((chat_id, parts[2].to_string()))
    } else {
        None
    }
}
