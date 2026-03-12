use serde::{Deserialize, Serialize};
use std::rc::Rc;

use crate::services::chat_service;
use crate::services::local_storage::{LocalChatIndex, LocalChatMessage, LocalDb, LocalDraftEntry};

#[derive(Debug, Clone, Serialize)]
struct SyncPullRequest {
    since_version: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct SyncEntryResponse {
    pub version: u64,
    pub entity_type: String,
    pub entity_path: String,
    pub action: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SyncChatIndex {
    pub id: String,
    pub model: String,
    pub message_ids: Vec<String>,
    #[serde(default)]
    pub version: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct SyncChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub reasoning: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub version: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct SyncData {
    pub messages: Vec<SyncChatMessage>,
    pub chats: Vec<SyncChatIndex>,
    pub drafts: Vec<SyncChatMessage>,
}

#[derive(Debug, Clone, Deserialize)]
struct SyncPullResponse {
    pub status: String,
    pub current_version: u64,
    pub entries: Vec<SyncEntryResponse>,
    pub data: SyncData,
}

#[derive(Debug, Clone, Serialize)]
struct SyncPushRequest {
    expected_version: u64,
    changes: Vec<SyncPushChange>,
}

#[derive(Debug, Clone, Serialize)]
struct SyncPushChange {
    entity_type: String,
    entity_id: String,
    chat_id: String,
    data: serde_json::Value,
    action: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SyncPushResponse {
    pub status: String,
    pub new_version: u64,
}

/// Information about a draft conflict between local and server versions.
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub draft_id: String,
    pub chat_id: String,
    pub local_content: String,
    pub server_content: String,
}

/// Result of a push operation.
#[derive(Debug)]
pub enum PushResult {
    Ok,
    Conflict(ConflictInfo),
}

pub struct SyncEngine {
    db: Rc<LocalDb>,
}

impl SyncEngine {
    pub fn new(db: Rc<LocalDb>) -> Self {
        SyncEngine { db }
    }

    /// Pull changes from server since our last known version.
    /// Returns true if new data was received.
    pub async fn pull(&self) -> Result<bool, String> {
        let since_version = self
            .db
            .get_last_synced_version()
            .await
            .map_err(|e| format!("Failed to get last synced version: {:?}", e))?;

        let resp: SyncPullResponse = chat_service::post_json(
            "/api/sync/pull",
            &SyncPullRequest { since_version },
        )
        .await?;

        if resp.current_version == since_version {
            return Ok(false);
        }

        // Store chats
        for chat in &resp.data.chats {
            let local_chat = LocalChatIndex {
                id: chat.id.clone(),
                model: chat.model.clone(),
                message_ids: chat.message_ids.iter().map(|id| id.to_string()).collect(),
                version: chat.version,
            };
            self.db
                .put_chat(&local_chat)
                .await
                .map_err(|e| format!("Failed to store chat: {:?}", e))?;
        }

        // Store messages — need to figure out chat_id from entries
        for msg in &resp.data.messages {
            let chat_id = find_chat_id_for_message(&resp.entries, &msg.id);
            let local_msg = LocalChatMessage {
                id: msg.id.clone(),
                chat_id,
                role: msg.role.clone(),
                content: msg.content.clone(),
                reasoning: msg.reasoning.clone(),
                created_at: msg.created_at.clone(),
                version: msg.version,
            };
            self.db
                .put_message(&local_msg)
                .await
                .map_err(|e| format!("Failed to store message: {:?}", e))?;
        }

        // Store drafts
        for draft in &resp.data.drafts {
            let chat_id = find_chat_id_for_draft(&resp.entries, &draft.id);
            let local_draft = LocalDraftEntry {
                id: draft.id.clone(),
                chat_id,
                content: draft.content.clone(),
                model: String::new(),
                version: draft.version,
            };
            self.db
                .put_draft(&local_draft)
                .await
                .map_err(|e| format!("Failed to store draft: {:?}", e))?;
        }

        // Handle deletions from entries
        for entry in &resp.entries {
            if entry.action == "Deleted" && entry.entity_type == "Draft" {
                if let Some(draft_id) = extract_last_segment(&entry.entity_path) {
                    let _ = self.db.delete_draft(&draft_id).await;
                }
            }
        }

        self.db
            .set_last_synced_version(resp.current_version)
            .await
            .map_err(|e| format!("Failed to set synced version: {:?}", e))?;

        Ok(true)
    }

    /// Push local drafts to server. Returns conflict info if there's a version mismatch.
    pub async fn push_drafts(&self, chat_id: &str) -> Result<PushResult, String> {
        let drafts = self
            .db
            .get_drafts_for_chat(chat_id)
            .await
            .map_err(|e| format!("Failed to get local drafts: {:?}", e))?;

        if drafts.is_empty() {
            return Ok(PushResult::Ok);
        }

        // Save local draft content before attempting push (pull may overwrite it)
        let local_drafts: Vec<LocalDraftEntry> = drafts.clone();

        let version = self
            .db
            .get_last_synced_version()
            .await
            .map_err(|e| format!("Failed to get synced version: {:?}", e))?;

        let changes: Vec<SyncPushChange> = drafts
            .iter()
            .map(|d| SyncPushChange {
                entity_type: "Draft".to_string(),
                entity_id: d.id.clone(),
                chat_id: d.chat_id.clone(),
                data: serde_json::json!({
                    "content": d.content,
                    "model": d.model,
                }),
                action: "Updated".to_string(),
            })
            .collect();

        let result: Result<SyncPushResponse, chat_service::ApiError> =
            chat_service::post_json_typed(
                "/api/sync/push",
                &SyncPushRequest {
                    expected_version: version,
                    changes,
                },
            )
            .await;

        match result {
            Ok(resp) => {
                self.db
                    .set_last_synced_version(resp.new_version)
                    .await
                    .map_err(|e| format!("Failed to update synced version: {:?}", e))?;
                Ok(PushResult::Ok)
            }
            Err(err) if err.is_version_conflict() => {
                // Pull server state to get the server's version of the draft
                let _ = self.pull().await;

                // Now IndexedDB has server versions. Get server draft content.
                let server_drafts = self
                    .db
                    .get_drafts_for_chat(chat_id)
                    .await
                    .map_err(|e| format!("Failed to get server drafts: {:?}", e))?;

                // Find the first conflicting draft
                if let Some(local_draft) = local_drafts.first() {
                    let server_content = server_drafts
                        .iter()
                        .find(|d| d.id == local_draft.id)
                        .map(|d| d.content.clone())
                        .unwrap_or_default();

                    // If content is the same, no real conflict — just version mismatch.
                    // Re-push with updated version (non-recursive).
                    if server_content == local_draft.content {
                        // Restore local draft in IndexedDB
                        let _ = self.db.put_draft(local_draft).await;
                        // Try push again with updated version
                        return self.try_push_drafts_once(chat_id).await;
                    }

                    // Restore local draft in IndexedDB so user can see it
                    let _ = self.db.put_draft(local_draft).await;

                    Ok(PushResult::Conflict(ConflictInfo {
                        draft_id: local_draft.id.clone(),
                        chat_id: local_draft.chat_id.clone(),
                        local_content: local_draft.content.clone(),
                        server_content,
                    }))
                } else {
                    Ok(PushResult::Ok)
                }
            }
            Err(err) => Err(err.message),
        }
    }

    /// Non-recursive push attempt. Returns error on any failure including conflict.
    async fn try_push_drafts_once(&self, chat_id: &str) -> Result<PushResult, String> {
        let drafts = self
            .db
            .get_drafts_for_chat(chat_id)
            .await
            .map_err(|e| format!("Failed to get local drafts: {:?}", e))?;

        if drafts.is_empty() {
            return Ok(PushResult::Ok);
        }

        let version = self
            .db
            .get_last_synced_version()
            .await
            .map_err(|e| format!("Failed to get synced version: {:?}", e))?;

        let changes: Vec<SyncPushChange> = drafts
            .iter()
            .map(|d| SyncPushChange {
                entity_type: "Draft".to_string(),
                entity_id: d.id.clone(),
                chat_id: d.chat_id.clone(),
                data: serde_json::json!({
                    "content": d.content,
                    "model": d.model,
                }),
                action: "Updated".to_string(),
            })
            .collect();

        let resp: SyncPushResponse = chat_service::post_json(
            "/api/sync/push",
            &SyncPushRequest {
                expected_version: version,
                changes,
            },
        )
        .await?;

        self.db
            .set_last_synced_version(resp.new_version)
            .await
            .map_err(|e| format!("Failed to update synced version: {:?}", e))?;

        Ok(PushResult::Ok)
    }

    /// Push a resolved draft to server. Saves to IndexedDB first, then pushes.
    pub async fn push_resolved_draft(&self, draft: &LocalDraftEntry) -> Result<(), String> {
        self.db
            .put_draft(draft)
            .await
            .map_err(|e| format!("Failed to save resolved draft: {:?}", e))?;

        match self.push_drafts(&draft.chat_id).await? {
            PushResult::Ok => Ok(()),
            PushResult::Conflict(_) => Err("Conflict persists after resolution. Please try again.".into()),
        }
    }

    /// Full sync cycle: pull then push.
    pub async fn sync(&self, chat_id: &str) -> Result<bool, String> {
        let had_changes = self.pull().await?;
        self.push_drafts(chat_id).await?;
        Ok(had_changes)
    }

    /// Bootstrap: called on new device, pulls everything (since_version: 0).
    pub async fn bootstrap(&self) -> Result<(), String> {
        self.db
            .set_last_synced_version(0)
            .await
            .map_err(|e| format!("Failed to reset version: {:?}", e))?;
        self.pull().await?;
        Ok(())
    }
}

/// Extract chat_id from a sync entry path like "chats/{chat_id}/messages/{msg_id}"
fn find_chat_id_for_message(entries: &[SyncEntryResponse], msg_id: &str) -> String {
    for entry in entries {
        if entry.entity_type == "Message" && entry.entity_path.contains(msg_id) {
            if let Some(chat_id) = extract_chat_id_from_path(&entry.entity_path) {
                return chat_id;
            }
        }
    }
    String::new()
}

fn find_chat_id_for_draft(entries: &[SyncEntryResponse], draft_id: &str) -> String {
    for entry in entries {
        if entry.entity_type == "Draft" && entry.entity_path.contains(draft_id) {
            if let Some(chat_id) = extract_chat_id_from_path(&entry.entity_path) {
                return chat_id;
            }
        }
    }
    String::new()
}

fn extract_chat_id_from_path(path: &str) -> Option<String> {
    // "chats/{chat_id}/messages/{msg_id}" or "chats/{chat_id}/drafts/{draft_id}"
    let stripped = path.strip_prefix("chats/")?;
    let slash_pos = stripped.find('/')?;
    Some(stripped[..slash_pos].to_string())
}

fn extract_last_segment(path: &str) -> Option<String> {
    path.rsplit('/').next().map(|s| s.to_string())
}
