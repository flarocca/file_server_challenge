use async_trait::async_trait;
use file_server_library::models::Hash32;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct FileMerkleTreeRow {
    pub id: Uuid,
    pub order: Vec<String>,
    pub files: HashMap<String, Hash32>,
    pub leaf_hashes: Vec<Hash32>,
    pub root: Option<Hash32>,
}

#[async_trait]
pub trait FileRepository: Send + Sync {
    async fn get(&self, id: Uuid) -> anyhow::Result<Option<FileMerkleTreeRow>>;
    async fn insert(&self, tree: FileMerkleTreeRow) -> anyhow::Result<()>;
    async fn update(&self, tree: FileMerkleTreeRow) -> anyhow::Result<()>;
}

#[cfg(feature = "in-memory")]
mod in_memory;
#[cfg(feature = "in-memory")]
pub use in_memory::InMemoryFileRepository;

#[cfg(feature = "persistent")]
mod clickhouse;
#[cfg(feature = "persistent")]
pub use clickhouse::{ClickhouseConfig, ClickhouseFileRepository};

// ChatGPT trick
#[cfg(all(feature = "in-memory", feature = "persistent"))]
compile_error!("Enable only one repo feature: `in-memory` OR `persistent`.");

#[cfg(not(any(feature = "in-memory", feature = "persistent")))]
compile_error!("Enable at least one repo feature: `in-memory` or `persistent`.");
