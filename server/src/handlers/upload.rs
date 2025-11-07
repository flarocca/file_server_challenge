use crate::{
    errors::ServerError,
    handlers::{requests::UploadMetadataRequest, responses::FileMetadataResponse},
    server::ServerState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use bytes::Bytes;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/{id}/upload",
    tag = "Upload File to File Tree",
    description = "Upload a file to the specified File Tree",
    params(
        ("id" = Uuid, Path, description = "File Tree ID"),
        ("name" = String, Query, description = "Name of the file to upload"),
        ("index" = usize, Query, description = "Index of the file within the File Tree"),
    ),
    request_body(
        content = inline(String),
        content_type = "application/octet-stream",
        description = "Raw file bytes"
    ),
    responses(
        (status = 200, description = "File Tree upload initiated", body = FileMetadataResponse),
        (status = 404, description = "File Tree not found"),
        (status = 409, description = "File already exists"),
        (status = 500, description = "Internal Server Error"),
    ),
)]
#[instrument(skip(state, metadata, body), fields(id = %id))]
pub async fn upload(
    State(state): State<Arc<ServerState>>,
    Query(metadata): Query<UploadMetadataRequest>,
    Path(id): Path<Uuid>,
    body: Bytes,
) -> Result<impl IntoResponse, ServerError> {
    let encoded_hash = state
        .file_service()
        .upload_file(id, metadata.clone().into(), body)
        .await
        .map_err(|e| {
            error!(
                "Failed to upload file with index {} and name {}: {:?}",
                metadata.index, metadata.name, e
            );
            ServerError::from(e)
        })?;

    Ok(Json(FileMetadataResponse {
        name: metadata.name.to_owned(),
        index: metadata.index,
        encoded_hash,
    }))
}
