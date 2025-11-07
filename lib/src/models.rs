use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// This struct is a wrapper around a 32-byte array representing a SHA-256 hash.
// The intention is to abstract implementation details behind something more meaningful.
// TODO: Allow different implementations of hash functions via features
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct Hash32([u8; 32]);

impl Hash32 {
    pub fn empty() -> Self {
        Hash32([0u8; 32])
    }

    pub fn hash(data: &[u8]) -> Hash32 {
        let digest = Sha256::digest(data);
        Hash32(digest.into())
    }

    pub fn to_hex(self) -> String {
        hex::encode(self)
    }

    pub fn from_hex(hash: &str) -> Result<Self, &'static str> {
        let Ok(bytes) = hex::decode(hash) else {
            return Err("Invalid hex string");
        };

        let Ok(hash) = Hash32::try_from(bytes.as_slice()) else {
            return Err("Invalid hash length");
        };

        Ok(hash)
    }
}

impl From<(&Hash32, &Hash32)> for Hash32 {
    fn from((left, right): (&Hash32, &Hash32)) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(left.0);
        hasher.update(right.0);

        Self(hasher.finalize().into())
    }
}

impl From<[u8; 32]> for Hash32 {
    fn from(x: [u8; 32]) -> Self {
        Hash32(x)
    }
}

impl From<Hash32> for [u8; 32] {
    fn from(x: Hash32) -> Self {
        x.0
    }
}

impl TryFrom<&[u8]> for Hash32 {
    type Error = &'static str;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 32 {
            return Err("Invalid length: expected 32 bytes");
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(slice);

        Ok(Hash32(array))
    }
}

impl TryFrom<Vec<u8>> for Hash32 {
    type Error = &'static str;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        Hash32::try_from(vec.as_slice())
    }
}

impl AsRef<[u8]> for Hash32 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Side {
    Left,
    Right,
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Left => write!(f, "Left"),
            Side::Right => write!(f, "Right"),
        }
    }
}

// My original aproach was to use -1 and 1 for sides (left and right respectively, that
// is how I used to manage similar structures back in University using C)
// ChatGPT suggested an enum called Direction instead, which is more idiomatic in Rust
// I ended up using the enum suggested from ChatGPT but keeping Side as name
// cause it is more semantically correct.
#[derive(Clone, Serialize, Deserialize)]
pub struct ProofStep {
    pub side: Side,
    pub hash: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Proof {
    pub leaf_hash: String,
    pub steps: Vec<ProofStep>,
}
