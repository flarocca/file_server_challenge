use crate::{
    errors::ServerError, handlers::responses::InitiateUploadResponse, server::ServerState,
};
use axum::{Json, extract::State, response::IntoResponse};
use reqwest::StatusCode;
use std::sync::Arc;
use tracing::{error, instrument};

#[utoipa::path(
    post,
    path = "/initiate",
    description = "Initiate a new File Tree upload",
    tag = "Initiate File Tree Upload",
    responses(
        (status = 201, description = "File Tree upload initiated", body = InitiateUploadResponse),
        (status = 500, description = "Internal Server Error"),
    ),
)]
#[instrument(skip(state))]
pub async fn initiate(
    State(state): State<Arc<ServerState>>,
) -> Result<impl IntoResponse, ServerError> {
    let id = state.file_service().initiate().await.map_err(|e| {
        error!("Failed to initiate upload: {:?}", e);
        ServerError::from(e)
    })?;

    Ok((StatusCode::CREATED, Json(InitiateUploadResponse { id })))
}
