use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::repositories::{FileMerkleTreeRow, FileRepository};

#[derive(Default)]
pub struct InMemoryFileRepository {
    file_trees: Mutex<HashMap<Uuid, FileMerkleTreeRow>>,
}

#[async_trait]
impl FileRepository for InMemoryFileRepository {
    async fn get(&self, id: Uuid) -> anyhow::Result<Option<FileMerkleTreeRow>> {
        let file_trees = self.file_trees.lock().await;
        Ok(file_trees.get(&id).cloned())
    }

    async fn insert(&self, tree: FileMerkleTreeRow) -> anyhow::Result<()> {
        let mut file_trees = self.file_trees.lock().await;
        file_trees.insert(tree.id, tree);
        Ok(())
    }

    async fn update(&self, tree: FileMerkleTreeRow) -> anyhow::Result<()> {
        let mut file_trees = self.file_trees.lock().await;
        file_trees.insert(tree.id, tree);
        Ok(())
    }
}
