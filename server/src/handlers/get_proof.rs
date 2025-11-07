use crate::{errors::ServerError, handlers::responses::ProofResponse, server::ServerState};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/{id}/proof/{index}",
    tag = "Get Proof from File Tree",
    description = "Retrieve the proof for a file in the specified File Tree",
    params(
        ("id" = Uuid, Path, description = "File Tree ID"),
        ("index" = usize, Path, description = "File index within the File Tree"),
    ),
    responses(
        (status = 200, description = "File Tree upload initiated", body = ProofResponse),
        (status = 404, description = "File Tree not found"),
        (status = 500, description = "Internal Server Error"),
    ),
)]
#[instrument(skip(state), fields(id = %id, index = %index))]
pub async fn get_proof(
    State(state): State<Arc<ServerState>>,
    Path((id, index)): Path<(Uuid, usize)>,
) -> Result<impl IntoResponse, ServerError> {
    let proof = state
        .file_service()
        .get_proof(id, index)
        .await
        .map_err(|e| {
            error!("Failed to get proof for file {}: {:?}", index, e);
            ServerError::from(e)
        })?;

    Ok(Json(ProofResponse::from(proof)))
}
