use crate::repositories::FileStorage;
use anyhow::Context;
use async_trait::async_trait;
use aws_sdk_s3::{Client, config::Builder, error::SdkError, primitives::ByteStream};
use config::Config;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub struct S3Config {
    pub bucket: String,
}

impl S3Config {
    const CONFIG_PREFIX: &'static str = "S3";

    pub fn load_from_env() -> anyhow::Result<Self> {
        Config::builder()
            .add_source(config::Environment::with_prefix(Self::CONFIG_PREFIX).separator("__"))
            .build()
            .unwrap()
            .try_deserialize::<S3Config>()
            .map_err(|e| anyhow::anyhow!("failed to load S3 Configuration: {}", e))
    }
}

#[derive(Clone)]
pub struct S3FileStorage {
    client: Client,
    bucket: String,
}

impl S3FileStorage {
    pub fn from_client(client: Client, bucket: String) -> Self {
        Self { client, bucket }
    }

    pub async fn load_from_env() -> anyhow::Result<Self> {
        let config = S3Config::load_from_env()?;
        let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

        // ChatGPT trick: This next line here was added to make it work with localstack.
        let conf = Builder::from(&aws_config).force_path_style(true).build();
        let client = Client::from_conf(conf);

        Ok(Self {
            client,
            bucket: config.bucket,
        })
    }

    fn key(&self, id: Uuid, name: &str) -> String {
        format!("{id}/{name}")
    }
}

#[async_trait]
#[async_trait]
impl FileStorage for S3FileStorage {
    async fn get_file_content(&self, id: Uuid, name: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let key = self.key(id, name);

        match self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(out) => {
                let bytes = out
                    .body
                    .collect()
                    .await
                    .with_context(|| "failed to read S3 body")?
                    .into_bytes()
                    .to_vec();

                Ok(Some(bytes))
            }
            Err(e) => {
                if let SdkError::ServiceError(se) = &e
                    && se.raw().status().as_u16() == 404
                {
                    return Ok(None);
                }
                Err(anyhow::anyhow!(e))
                    .with_context(|| format!("get_object {}/{}", self.bucket, key))
            }
        }
    }

    async fn insert_file_content(
        &self,
        id: Uuid,
        name: &str,
        content: &[u8],
    ) -> anyhow::Result<()> {
        let key = self.key(id, name);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(content.to_vec()))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!(e))
            .with_context(|| format!("put_object {}/{}", self.bucket, key))?;

        Ok(())
    }
}
