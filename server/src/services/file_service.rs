use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use file_server_library::{
    CustomMerkleTree,
    models::{Hash32, Proof},
};
use tracing::error;
use uuid::Uuid;

use crate::{
    models::{FileContent, FileMerkleTree, FileMetadata},
    repositories::{FileRepository, FileStorage},
};

#[derive(Debug)]
pub enum FileServiceError {
    FileNotFound,
    FileIndexNotFound,
    FileAlreadyExists,
    StorageError(String),
}

#[async_trait]
pub trait FileService: Send + Sync {
    async fn get_file_content(
        &self,
        id: Uuid,
        index: usize,
    ) -> Result<FileContent, FileServiceError>;
    async fn get_proof(&self, id: Uuid, index: usize) -> Result<Proof, FileServiceError>;
    async fn initiate(&self) -> Result<Uuid, FileServiceError>;
    async fn upload_file(
        &self,
        id: Uuid,
        metadata: FileMetadata,
        content: Bytes,
    ) -> Result<String, FileServiceError>;
    async fn complete(&self, id: Uuid) -> Result<String, FileServiceError>;
}

pub struct FileServiceImpl {
    file_repository: Arc<dyn FileRepository>,
    file_storage: Arc<dyn FileStorage>,
}

impl FileServiceImpl {
    pub fn new(
        file_repository: Arc<dyn FileRepository>,
        file_storage: Arc<dyn FileStorage>,
    ) -> Self {
        Self {
            file_repository,
            file_storage,
        }
    }

    pub async fn get_file_content(
        &self,
        id: Uuid,
        index: usize,
    ) -> Result<FileContent, FileServiceError> {
        let file_tree = self.get_file_tree(id).await?;

        let file_name = file_tree
            .get_file_name_by_index(index)
            .ok_or(FileServiceError::FileNotFound)?;

        let content = self
            .file_storage
            .get_file_content(id, &file_name)
            .await
            .map_err(|e| FileServiceError::StorageError(e.to_string()))?
            .ok_or(FileServiceError::FileIndexNotFound)?;

        Ok(content)
    }

    pub async fn get_proof(&self, id: Uuid, index: usize) -> Result<Proof, FileServiceError> {
        let file_tree = self.get_file_tree(id).await?;

        let tree = CustomMerkleTree::new(file_tree.leafs());
        let proof = tree.proof(index);

        Ok(proof)
    }

    pub async fn initiate(&self) -> Result<Uuid, FileServiceError> {
        let file_tree = FileMerkleTree::default();

        self.file_repository
            .insert(file_tree.clone().into())
            .await
            .map_err(|e| FileServiceError::StorageError(e.to_string()))?;

        Ok(file_tree.id())
    }

    pub async fn upload_file(
        &self,
        id: Uuid,
        metadata: FileMetadata,
        content: Bytes,
    ) -> Result<String, FileServiceError> {
        let mut file_tree = self.get_file_tree(id).await?;

        if file_tree.contains_file(&metadata.name) {
            return Err(FileServiceError::FileAlreadyExists);
        }

        self.file_storage
            .insert_file_content(id, &metadata.name, &content)
            .await
            .map_err(|e| FileServiceError::StorageError(e.to_string()))?;

        let hash = Hash32::hash(&content);
        file_tree.add(metadata.index, &metadata.name, &hash);

        self.file_repository
            .update(file_tree.clone().into())
            .await
            .map_err(|e| FileServiceError::StorageError(e.to_string()))?;

        Ok(hex::encode(hash))
    }

    pub async fn complete(&self, id: Uuid) -> Result<String, FileServiceError> {
        println!("Completing file tree with id: {}", id);
        let mut file_tree = self.get_file_tree(id).await?;

        let tree = CustomMerkleTree::new(file_tree.leafs());
        let root_hash = tree.root();
        file_tree.complete(root_hash);

        self.file_repository
            .update(file_tree.into())
            .await
            .map_err(|e| FileServiceError::StorageError(e.to_string()))?;

        Ok(root_hash.to_hex())
    }
}

#[async_trait]
impl FileService for FileServiceImpl {
    async fn get_file_content(
        &self,
        id: Uuid,
        index: usize,
    ) -> Result<FileContent, FileServiceError> {
        let file_tree = self.get_file_tree(id).await?;

        let file_name = file_tree
            .get_file_name_by_index(index)
            .ok_or(FileServiceError::FileNotFound)?;

        let content = self
            .file_storage
            .get_file_content(id, &file_name)
            .await
            .map_err(|e| {
                error!("Failed to insert file tree: {}", e);
                FileServiceError::StorageError(e.to_string())
            })?
            .ok_or(FileServiceError::FileIndexNotFound)?;

        Ok(content)
    }

    async fn get_proof(&self, id: Uuid, index: usize) -> Result<Proof, FileServiceError> {
        let file_tree = self.get_file_tree(id).await?;

        let tree = CustomMerkleTree::new(file_tree.leafs());
        let proof = tree.proof(index);

        Ok(proof)
    }

    async fn initiate(&self) -> Result<Uuid, FileServiceError> {
        let file_tree = FileMerkleTree::default();

        println!(
            "Initiating new file tree upload with id: {}",
            file_tree.id()
        );

        self.file_repository
            .insert(file_tree.clone().into())
            .await
            .map_err(|e| {
                error!("Failed to insert file tree: {}", e);
                FileServiceError::StorageError(e.to_string())
            })?;

        Ok(file_tree.id())
    }

    async fn upload_file(
        &self,
        id: Uuid,
        metadata: FileMetadata,
        content: Bytes,
    ) -> Result<String, FileServiceError> {
        let mut file_tree = self.get_file_tree(id).await?;

        if file_tree.contains_file(&metadata.name) {
            return Err(FileServiceError::FileAlreadyExists);
        }

        self.file_storage
            .insert_file_content(id, &metadata.name, &content)
            .await
            .map_err(|e| {
                error!("Failed to insert file tree: {}", e);
                FileServiceError::StorageError(e.to_string())
            })?;

        let hash = Hash32::hash(&content);
        file_tree.add(metadata.index, &metadata.name, &hash);

        self.file_repository
            .update(file_tree.clone().into())
            .await
            .map_err(|e| {
                error!("Failed to insert file tree: {}", e);
                FileServiceError::StorageError(e.to_string())
            })?;

        Ok(hex::encode(hash))
    }

    async fn complete(&self, id: Uuid) -> Result<String, FileServiceError> {
        println!("Completing file tree with id: {}", id);
        let mut file_tree = self.get_file_tree(id).await?;

        let tree = CustomMerkleTree::new(file_tree.leafs());
        let root_hash = tree.root();
        file_tree.complete(root_hash);

        self.file_repository
            .update(file_tree.into())
            .await
            .map_err(|e| {
                error!("Failed to insert file tree: {}", e);
                FileServiceError::StorageError(e.to_string())
            })?;

        Ok(root_hash.to_hex())
    }
}

impl FileServiceImpl {
    async fn get_file_tree(&self, id: Uuid) -> Result<FileMerkleTree, FileServiceError> {
        let file_tree: FileMerkleTree = self
            .file_repository
            .get(id)
            .await
            .map_err(|e| FileServiceError::StorageError(e.to_string()))?
            .ok_or(FileServiceError::FileNotFound)?
            .into();

        Ok(file_tree)
    }
}
