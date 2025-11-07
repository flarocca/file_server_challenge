// This file is part of the template, response structs are defined here.

use file_server_library::models::{Proof, ProofStep};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct FileMetadataResponse {
    pub name: String,
    pub index: usize,
    pub encoded_hash: String,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct InitiateUploadResponse {
    pub id: Uuid,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct FinalUploadResponse {
    pub root_hex: String,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct ProofResponse {
    pub leaf_hash: String,
    pub steps: Vec<ProofStepResponse>,
}

impl From<Proof> for ProofResponse {
    fn from(proof: Proof) -> Self {
        Self {
            leaf_hash: proof.leaf_hash,
            steps: proof
                .steps
                .into_iter()
                .map(ProofStepResponse::from)
                .collect(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct ProofStepResponse {
    pub side: String,
    pub hash: String,
}

impl From<ProofStep> for ProofStepResponse {
    fn from(step: ProofStep) -> Self {
        Self {
            side: step.side.to_string(),
            hash: step.hash,
        }
    }
}
