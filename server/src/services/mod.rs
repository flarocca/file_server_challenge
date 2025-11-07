mod file_service;
pub use file_service::{FileService, FileServiceError, FileServiceImpl};

use crate::repositories::{FileRepository, FileStorage};
use std::sync::Arc;

// Template function to initialize services.
// In a more complex project, this should return a IoC container instead with
// all services registered.
pub async fn init_services(
    file_repository: Arc<dyn FileRepository>,
    file_storage: Arc<dyn FileStorage>,
) -> anyhow::Result<Arc<dyn FileService>> {
    let file_service =
        Arc::new(FileServiceImpl::new(file_repository, file_storage)) as Arc<dyn FileService>;
    Ok(file_service)
}
