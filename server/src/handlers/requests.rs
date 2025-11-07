// This file is part of the template, requests structs are defined here.

use crate::models::FileMetadata;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Clone, Deserialize, ToSchema)]
pub struct UploadMetadataRequest {
    pub name: String,
    pub index: usize,
}

impl From<FileMetadata> for UploadMetadataRequest {
    fn from(meta: FileMetadata) -> Self {
        Self {
            name: meta.name,
            index: meta.index,
        }
    }
}

impl From<UploadMetadataRequest> for FileMetadata {
    fn from(val: UploadMetadataRequest) -> Self {
        FileMetadata {
            name: val.name,
            index: val.index,
        }
    }
}
