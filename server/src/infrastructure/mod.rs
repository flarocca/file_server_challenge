// This file is part of the template, usually, this does not noeed to be modified.

mod authentication;
mod helpers;
mod tracing;

pub use crate::infrastructure::authentication::*;
pub use crate::infrastructure::tracing::*;
pub use helpers::*;

pub async fn init_infrastructure() {
    dotenv::dotenv().ok();
    init_tracing();
}
