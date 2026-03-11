use actix_web::{FromRequest, HttpRequest};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

use crate::models::chat::{ChatId, ChatIndex, ChatMeta, ChatSummary};
use crate::providers::chat_storage::{ChatStorage, ChatStorageError};
use crate::providers::personalized_file_storage::PersonalizedFileStorage;

pub struct PersonalizedChatStorage {
    file_storage: PersonalizedFileStorage,
}

impl PersonalizedChatStorage {
    pub fn get_chat_storage(&self, chat_id: ChatId) -> ChatStorage {
        ChatStorage::new(self.file_storage.clone(), chat_id)
    }

    pub async fn list_chats(&self) -> Result<Vec<ChatSummary>, ChatStorageError> {
        let keys = self
            .file_storage
            .list("chats/")
            .await
            .map_err(ChatStorageError::S3)?;

        let mut summaries = Vec::new();
        for key in &keys {
            if key.ends_with("/chat-meta.yaml") {
                // Extract chat_id from path: chats/{chat_id}/chat-meta.yaml
                let chat_id = key
                    .strip_prefix("chats/")
                    .and_then(|s| s.strip_suffix("/chat-meta.yaml"))
                    .and_then(|s| Uuid::parse_str(s).ok());

                let chat_id = match chat_id {
                    Some(id) => id,
                    None => continue,
                };

                let meta_data = self
                    .file_storage
                    .get(key)
                    .await
                    .map_err(ChatStorageError::S3)?;
                let meta: ChatMeta = serde_yaml::from_str(&String::from_utf8(meta_data)?)?;

                let index_key = format!("chats/{}/chat.yaml", chat_id);
                let index_data = self
                    .file_storage
                    .get(&index_key)
                    .await
                    .map_err(ChatStorageError::S3)?;
                let index: ChatIndex = serde_yaml::from_str(&String::from_utf8(index_data)?)?;

                summaries.push(ChatSummary {
                    id: chat_id,
                    model: index.model,
                    created_at: meta.created_at,
                });
            }
        }

        summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(summaries)
    }

    pub async fn delete_chat(&self, chat_id: ChatId) -> Result<(), ChatStorageError> {
        let prefix = format!("chats/{}/", chat_id);
        let keys = self
            .file_storage
            .list(&prefix)
            .await
            .map_err(ChatStorageError::S3)?;

        for key in &keys {
            self.file_storage
                .delete(key)
                .await
                .map_err(ChatStorageError::S3)?;
        }
        Ok(())
    }
}

impl FromRequest for PersonalizedChatStorage {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let file_storage_fut = PersonalizedFileStorage::from_request(req, payload);

        Box::pin(async move {
            let file_storage = file_storage_fut.await?;
            Ok(PersonalizedChatStorage { file_storage })
        })
    }
}
