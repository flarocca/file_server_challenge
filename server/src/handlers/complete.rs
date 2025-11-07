use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use tracing::{error, instrument};
use uuid::Uuid;

use crate::{errors::ServerError, handlers::responses::FinalUploadResponse, server::ServerState};

#[utoipa::path(
    post,
    path = "/{id}/complete",
    tag = "Complete File Tree Upload",
    description = "Complete the file tree upload and get the root hash",
    params(
        ("id" = Uuid, Path, description = "File Tree ID"),
    ),
    responses(
        (status = 200, description = "File Tree upload initiated", body = FinalUploadResponse),
        (status = 404, description = "File Tree not found"),
        (status = 500, description = "Internal Server Error"),
    ),
)]
#[instrument(skip(state), fields(id = %id))]
pub async fn complete(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServerError> {
    let encoded_root_hash = state.file_service().complete(id).await.map_err(|e| {
        error!("Failed to complete upload {}: {:?}", id, e);
        ServerError::from(e)
    })?;

    Ok(Json(FinalUploadResponse {
        root_hex: encoded_root_hash,
    }))
}
