use crate::repositories::FileMerkleTreeRow;
use file_server_library::models::Hash32;
use std::collections::HashMap;
use uuid::Uuid;

pub type FileName = String;
pub type FileContent = Vec<u8>;

pub struct FileMetadata {
    pub name: String,
    pub index: usize,
}

#[derive(Clone)]
pub struct FileMerkleTree {
    id: Uuid,
    order: Vec<FileName>,
    files: HashMap<FileName, Hash32>,
    leaf_hashes: Vec<Hash32>,
    root: Option<Hash32>,
}

impl Default for FileMerkleTree {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            order: Vec::new(),
            files: HashMap::new(),
            leaf_hashes: Vec::new(),
            root: None,
        }
    }
}

impl From<FileMerkleTreeRow> for FileMerkleTree {
    fn from(row: FileMerkleTreeRow) -> Self {
        Self {
            id: row.id,
            order: row.order,
            files: row.files,
            leaf_hashes: row.leaf_hashes,
            root: row.root,
        }
    }
}

impl From<FileMerkleTree> for FileMerkleTreeRow {
    fn from(val: FileMerkleTree) -> Self {
        FileMerkleTreeRow {
            id: val.id,
            order: val.order,
            files: val.files,
            leaf_hashes: val.leaf_hashes,
            root: val.root,
        }
    }
}

impl FileMerkleTree {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn leafs(&self) -> Vec<Hash32> {
        self.leaf_hashes.clone()
    }

    pub fn get_file_name_by_index(&self, index: usize) -> Option<String> {
        self.order.get(index).cloned()
    }

    pub fn contains_file(&self, name: &str) -> bool {
        self.files.contains_key(name)
    }

    pub fn add(&mut self, index: usize, name: &str, hash: &Hash32) {
        if self.order.len() <= index {
            self.order.resize(index + 1, String::new());
            self.leaf_hashes.resize(index + 1, Hash32::empty());
        }

        self.order[index] = name.to_owned();
        self.leaf_hashes[index] = *hash;

        self.root = None;
    }

    pub fn complete(&mut self, root: Hash32) {
        self.root = Some(root);
    }
}
