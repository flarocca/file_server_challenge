use bytes::Bytes;
use file_server_library::models::Proof;
use file_server_server::models::{FileContent, FileMetadata};
use file_server_server::services::FileService;
use file_server_server::services::FileServiceError;
use mockall::mock;
use uuid::Uuid;

mock! {
    pub FileServiceImpl {}

    #[async_trait::async_trait]
    impl FileService for FileServiceImpl {
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
}
