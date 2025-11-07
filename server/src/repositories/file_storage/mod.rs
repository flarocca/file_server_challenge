use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait FileStorage: Send + Sync {
    async fn get_file_content(&self, id: Uuid, name: &str) -> anyhow::Result<Option<Vec<u8>>>;
    async fn insert_file_content(&self, id: Uuid, name: &str, content: &[u8])
    -> anyhow::Result<()>;
}

#[cfg(feature = "in-memory")]
mod in_memory;
#[cfg(feature = "in-memory")]
pub use in_memory::InMemoryFileStorage;

#[cfg(feature = "persistent")]
mod s3;
#[cfg(feature = "persistent")]
pub use s3::{S3Config, S3FileStorage};

// ChatGPT trick
#[cfg(all(feature = "in-memory", feature = "persistent"))]
compile_error!("Enable only one repo feature: `in-memory` OR `persistent`.");

#[cfg(not(any(feature = "in-memory", feature = "persistent")))]
compile_error!("Enable at least one repo feature: `in-memory` or `persistent`.");
