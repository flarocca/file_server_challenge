// This file is part of the template, new routes modules are defined here
// and added to the router.

use crate::server::ServerState;
use std::sync::Arc;
use utoipa_axum::{router::OpenApiRouter, routes};

mod complete;
mod get_file;
mod get_proof;
mod initiate;
pub mod requests;
pub mod responses;
mod upload;

pub fn router(state: Arc<ServerState>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(upload::upload))
        .routes(routes!(complete::complete))
        .routes(routes!(initiate::initiate,))
        .routes(routes!(get_proof::get_proof))
        .routes(routes!(get_file::get_file))
        .with_state(state)
}
