use crate::{errors::ServerError, server::ServerState};
use axum::extract::{Path, State};
use bytes::Bytes;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/{id}/file/{index}",
    tag = "Get File from File Tree",
    description = "Retrieve the raw file bytes from the specified File Tree",
    params(
        ("id" = Uuid, Path, description = "File Tree ID"),
        ("index" = usize, Path, description = "File index within the File Tree"),
    ),
    responses(
        (status = 200, description = "File Tree upload initiated", body = String, content_type = "application/octet-stream"),
        (status = 404, description = "File Tree not found"),
        (status = 500, description = "Internal Server Error"),
    ),
)]
#[instrument(skip(state), fields(id = %id, index = %index))]
pub async fn get_file(
    State(state): State<Arc<ServerState>>,
    Path((id, index)): Path<(Uuid, usize)>,
) -> Result<Bytes, ServerError> {
    let contents = state
        .file_service()
        .get_file_content(id, index)
        .await
        .map_err(|e| {
            error!("Failed to get file for file {}: {:?}", index, e);
            ServerError::from(e)
        })?;

    Ok(Bytes::from(contents))
}
