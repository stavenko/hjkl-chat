use actix_web::{web, FromRequest, HttpRequest};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::api::auth_extractor::AuthenticatedUser;
use crate::models::chat::{UserId, UserMemory};
use crate::providers::s3::{S3Provider, S3ProviderError};

#[derive(Clone)]
pub struct PersonalizedFileStorage {
    s3: Arc<S3Provider>,
    user_id: UserId,
}

impl PersonalizedFileStorage {
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    fn scoped_path(&self, path: &str) -> String {
        format!("{}/{}", self.user_id, path)
    }

    pub async fn put(&self, path: &str, data: &[u8]) -> Result<(), S3ProviderError> {
        let key = self.scoped_path(path);
        self.s3.put_object(&key, data).await
    }

    pub async fn get(&self, path: &str) -> Result<Vec<u8>, S3ProviderError> {
        let key = self.scoped_path(path);
        self.s3.get_object(&key).await
    }

    pub async fn list(&self, prefix: &str) -> Result<Vec<String>, S3ProviderError> {
        let scoped_prefix = self.scoped_path(prefix);
        let keys = self.s3.list_objects(&scoped_prefix).await?;
        let strip_prefix = format!("{}/", self.user_id);
        Ok(keys
            .into_iter()
            .map(|k| k.strip_prefix(&strip_prefix).unwrap_or(&k).to_string())
            .collect())
    }

    pub async fn delete(&self, path: &str) -> Result<(), S3ProviderError> {
        let key = self.scoped_path(path);
        self.s3.delete_object(&key).await
    }

    pub async fn exists(&self, path: &str) -> Result<bool, S3ProviderError> {
        let key = self.scoped_path(path);
        self.s3.object_exists(&key).await
    }

    pub async fn get_user_memory(&self) -> Result<Option<UserMemory>, S3ProviderError> {
        let path = "user-memory.yaml";
        if !self.exists(path).await? {
            return Ok(None);
        }
        let data = self.get(path).await?;
        let yaml_str = String::from_utf8(data)
            .map_err(|e| S3ProviderError::Other(e.to_string()))?;
        let memory: UserMemory = serde_yaml::from_str(&yaml_str)
            .map_err(|e| S3ProviderError::Other(e.to_string()))?;
        Ok(Some(memory))
    }

    pub async fn save_user_memory(&self, memory: &UserMemory) -> Result<(), S3ProviderError> {
        let yaml = serde_yaml::to_string(memory)
            .map_err(|e| S3ProviderError::Other(e.to_string()))?;
        self.put("user-memory.yaml", yaml.as_bytes()).await
    }
}

impl FromRequest for PersonalizedFileStorage {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let auth_fut = AuthenticatedUser::from_request(req, payload);
        let s3 = req.app_data::<web::Data<Arc<S3Provider>>>().cloned();

        Box::pin(async move {
            let user = auth_fut.await?;
            let s3 = s3.ok_or_else(|| {
                actix_web::error::ErrorInternalServerError("S3 provider not available")
            })?;

            Ok(PersonalizedFileStorage {
                s3: s3.get_ref().clone(),
                user_id: user.user_id,
            })
        })
    }
}
