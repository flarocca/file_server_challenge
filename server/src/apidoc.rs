// This file is part of the template, change the API documentation here.

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(info(
    title = "Custom File Server API",
    version = "1.0.0",
    description = "Custom File Server API"
))]
pub struct ApiDoc;
