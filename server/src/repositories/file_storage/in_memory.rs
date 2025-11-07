use crate::repositories::FileStorage;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryFileStorage {
    file_tree_contents: Mutex<HashMap<(Uuid, String), Vec<u8>>>,
}

#[async_trait]
impl FileStorage for InMemoryFileStorage {
    async fn insert_file_content(
        &self,
        id: Uuid,
        name: &str,
        content: &[u8],
    ) -> anyhow::Result<()> {
        let mut file_tree_contents = self.file_tree_contents.lock().await;

        file_tree_contents.insert((id, name.to_string()), content.to_vec());

        Ok(())
    }

    async fn get_file_content(&self, id: Uuid, name: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let file_tree_contents = self.file_tree_contents.lock().await;
        Ok(file_tree_contents.get(&(id, name.to_string())).cloned())
    }
}
