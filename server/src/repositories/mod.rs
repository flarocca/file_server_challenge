mod file_repository;
mod file_storage;

use std::sync::Arc;

#[cfg(feature = "persistent")]
pub use file_repository::{ClickhouseConfig, ClickhouseFileRepository};
pub use file_repository::{FileMerkleTreeRow, FileRepository};
pub use file_storage::FileStorage;
#[cfg(feature = "persistent")]
pub use file_storage::{S3Config, S3FileStorage};

// Template function to initialize repositories.
pub async fn init_repositories() -> anyhow::Result<(Arc<dyn FileRepository>, Arc<dyn FileStorage>)>
{
    // This is not part of the template repository, it just shows the advantages of the repository
    // structure and traits implemented in here. Since we can dockerize everything using clickhouse
    // images and loccalstack, it is unlikely that the in-memory version is needed.
    #[cfg(feature = "in-memory")]
    {
        use crate::repositories::{
            file_repository::InMemoryFileRepository, file_storage::InMemoryFileStorage,
        };

        let file_repository = Arc::new(InMemoryFileRepository::default());
        let file_storage = Arc::new(InMemoryFileStorage::default());

        Ok((file_repository, file_storage))
    }
    #[cfg(feature = "persistent")]
    {
        use crate::repositories::{
            file_repository::{ClickhouseConfig, ClickhouseFileRepository},
            file_storage::S3FileStorage,
        };

        let clickhouse_config = ClickhouseConfig::load_from_env()?;
        let file_repository = Arc::new(ClickhouseFileRepository::new(clickhouse_config));
        let file_storage = Arc::new(S3FileStorage::load_from_env().await?);

        Ok((file_repository, file_storage))
    }
}
