// I chose Clickhouse cause it is what I've been using recently and I have already boilerplate for
// it, otherwise I would have chosen something simpler or more popular (PostgreSQL, MongoDB, etc).
// In fact, there is a great ORM for PostgreSQL (Diesel) that would make this code much simpler and
// maintainable (migrations can be painful without a framework, sdk or crate).
// Having SQL queries in code as it is right now is less than ideal, so I very desirable todo would
// be to try a custom implementation.
use async_trait::async_trait;
use chrono::Utc;
use clickhouse::{Client, Row};
use config::Config;
use file_server_library::models::Hash32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::repositories::{FileMerkleTreeRow, FileRepository};

const FILE_TABLE_NAME: &str = "files";

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub struct ClickhouseConfig {
    pub database_url: String,
    pub database_name: String,
    pub username: String,
    pub password: String,
}

impl ClickhouseConfig {
    const CONFIG_PREFIX: &'static str = "CLICKHOUSE";

    pub fn load_from_env() -> anyhow::Result<Self> {
        Config::builder()
            .add_source(config::Environment::with_prefix(Self::CONFIG_PREFIX).separator("__"))
            .build()
            .unwrap()
            .try_deserialize::<ClickhouseConfig>()
            .map_err(|e| anyhow::anyhow!("failed to load ClickhouseConfig: {}", e))
    }
}

pub struct ClickhouseFileRepository {
    client: Client,
}

impl ClickhouseFileRepository {
    pub fn new(config: ClickhouseConfig) -> Self {
        let client = Client::default()
            .with_url(config.database_url.to_owned())
            .with_user(config.username.to_owned())
            .with_password(config.password.to_owned())
            .with_database(config.database_name.to_owned())
            .with_option("send_progress_in_http_headers", "1");

        Self::new_from_client(client)
    }

    pub fn new_from_client(client: Client) -> Self {
        Self { client }
    }
}

// This DTO is required to handle HashMaps, which are not supported natively by Clickhouse crate.
// Additionally, it is a great example of why it is important to keep a clear separation between
// the database layer and the service layer. That separation is achieved by not sharing same
// structs between layers.
#[derive(Serialize, Deserialize, Row)]
struct ClickhouseFileRow {
    #[serde(with = "clickhouse::serde::uuid")]
    id: Uuid,
    files_order: Vec<String>,
    files: Vec<(String, String)>,
    leaf_hashes: Vec<String>,
    root: Option<String>,
}

// ChatGPT snippet
impl From<FileMerkleTreeRow> for ClickhouseFileRow {
    fn from(x: FileMerkleTreeRow) -> Self {
        let files = x.files.into_iter().map(|(k, v)| (k, v.to_hex())).collect();
        let leaf_hashes = x.leaf_hashes.into_iter().map(|h| h.to_hex()).collect();
        let root = x.root.map(|h| h.to_hex());
        Self {
            id: x.id,
            files_order: x.order,
            files,
            leaf_hashes,
            root,
        }
    }
}

// ChatGPT snippet
impl TryFrom<ClickhouseFileRow> for FileMerkleTreeRow {
    type Error = anyhow::Error;

    fn try_from(row: ClickhouseFileRow) -> anyhow::Result<Self> {
        let files: HashMap<_, _> = row
            .files
            .into_iter()
            .map(|(k, v)| {
                Hash32::from_hex(&v)
                    .map(|h| (k, h))
                    .map_err(|e| anyhow::anyhow!("bad hash {v}: {e}"))
            })
            .collect::<anyhow::Result<_>>()?;

        let leaf_hashes = row
            .leaf_hashes
            .into_iter()
            .map(|v| Hash32::from_hex(&v).map_err(|e| anyhow::anyhow!("bad leaf hash {v}: {e}")))
            .collect::<anyhow::Result<Vec<_>>>()?;

        let root = match row.root {
            Some(s) => Some(Hash32::from_hex(&s).map_err(|_| anyhow::anyhow!("bad root"))?),
            None => None,
        };

        Ok(FileMerkleTreeRow {
            id: row.id,
            order: row.files_order,
            files,
            leaf_hashes,
            root,
        })
    }
}

#[async_trait]
impl FileRepository for ClickhouseFileRepository {
    async fn get(&self, id: Uuid) -> anyhow::Result<Option<FileMerkleTreeRow>> {
        let sql = format!(
            "SELECT 
                 id,
                 files_order,
                 files,
                 leaf_hashes,
                 root
               FROM {FILE_TABLE_NAME}
              WHERE id = ?",
        );
        let mut cursor = self
            .client
            .query(&sql)
            .bind(id)
            .fetch::<ClickhouseFileRow>()?;

        if let Some(row) = cursor.next().await? {
            Ok(Some(row.try_into()?))
        } else {
            Ok(None)
        }
    }

    async fn insert(&self, tree: FileMerkleTreeRow) -> anyhow::Result<()> {
        // TODO: validate there is no row for this id already
        let mut insert = self
            .client
            .clone()
            .with_option("mutations_sync", "1")
            .insert::<ClickhouseFileRow>(FILE_TABLE_NAME)
            .await?;

        insert.write(&tree.into()).await?;
        insert.end().await?;

        Ok(())
    }

    async fn update(&self, row: FileMerkleTreeRow) -> anyhow::Result<()> {
        // TODO: validate there is a row for this id
        let item: ClickhouseFileRow = row.into();

        let sql = format!(
            "ALTER TABLE {FILE_TABLE_NAME} UPDATE
                files_order = ?,
                files = ?,
                leaf_hashes = ?,
                root = ?,
                updated_at = ?
             WHERE id = ?",
        );

        self.client
            .clone()
            .with_option("mutations_sync", "1")
            .query(&sql)
            .bind(&item.files_order)
            .bind(item.files)
            .bind(item.leaf_hashes)
            .bind(&item.root)
            .bind(Utc::now().timestamp_millis().to_string())
            .bind(item.id)
            .execute()
            .await?;

        Ok(())
    }
}
