use aws_config::{Region, meta::region::RegionProviderChain};
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Credentials;

#[derive(thiserror::Error, Debug)]
pub enum S3ProviderError {
    #[error("AWS SDK error: {0}")]
    AwsSdk(String),
    #[allow(dead_code)]
    #[error("AWS config error: {0}")]
    AwsConfig(String),
    #[error("Byte stream error: {0}")]
    ByteStream(String),
}

pub type S3ProviderResult<T> = Result<T, S3ProviderError>;

pub struct S3Provider {
    pub client: Client,
    pub bucket: String,
}

impl S3Provider {
    pub async fn new(
        bucket: String,
        region: String,
        client_id: String,
        client_secret: String,
        host: String,
    ) -> S3ProviderResult<Self> {
        let region_provider = RegionProviderChain::first_try(
            Region::new(region),
        );

        let credentials = Credentials::new(client_id, client_secret, None, None, "static");

        let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region_provider)
            .credentials_provider(credentials)
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&sdk_config)
            .endpoint_url(&host)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(s3_config);

        Ok(S3Provider { client, bucket })
    }

    pub async fn put_object(
        &self,
        key: &str,
        body: &[u8],
    ) -> S3ProviderResult<()> {
        use aws_sdk_s3::primitives::ByteStream;
        
        self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(body.to_vec()))
            .send()
            .await
            .map_err(|e| S3ProviderError::AwsSdk(e.to_string()))?;

        Ok(())
    }

    pub async fn get_object(&self, key: &str) -> S3ProviderResult<Vec<u8>> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| S3ProviderError::AwsSdk(e.to_string()))?;

        let body_bytes = response.body.collect().await.map_err(|e| {
            S3ProviderError::ByteStream(e.to_string())
        })?;
        Ok(body_bytes.into_bytes().to_vec())
    }

    #[allow(dead_code)]
    pub async fn delete_object(&self, key: &str) -> S3ProviderResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| S3ProviderError::AwsSdk(e.to_string()))?;
        Ok(())
    }

    pub async fn object_exists(&self, key: &str) -> S3ProviderResult<bool> {
        match self.client.head_object().bucket(&self.bucket).key(key).send().await {
            Ok(_) => Ok(true),
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("404") || error_str.contains("NotFound") {
                    Ok(false)
                } else {
                    Err(S3ProviderError::AwsSdk(error_str))
                }
            }
        }
    }
}