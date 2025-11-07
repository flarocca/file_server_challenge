// These DTOs are exactly the same as the ones the server uses. One might be tempted to import the
// crate directly instead of duplicating the code. I disagree wit approach as it would create a
// coupling between client and server and the binary level.
// There are tools to create DTOs and ApiClients from OpenAPI specification that will simplify the
// maintenance (e.g. progenitor crate, didn't use it here cause I never tested it before)
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiateUploadResponse {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadataResponse {
    pub name: String,
    pub index: usize,
    pub encoded_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalUploadResponse {
    pub root_hex: String,
}
