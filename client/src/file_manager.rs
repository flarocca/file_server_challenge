use std::{fs, path::PathBuf};

use uuid::Uuid;

pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub data: Vec<u8>,
}

pub struct FileManagerArgs {
    pub roots_storage_path: PathBuf,
    pub files_storage_path: PathBuf,
}

pub struct FileManager {
    args: FileManagerArgs,
}

impl FileManager {
    pub fn new(args: FileManagerArgs) -> Self {
        Self { args }
    }

    pub fn load_files(&self) -> anyhow::Result<Vec<FileEntry>> {
        let files: Vec<_> = fs::read_dir(self.args.files_storage_path.clone())?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .collect();

        if files.is_empty() {
            return Ok(Vec::new());
        }

        let mut result = Vec::with_capacity(files.len());

        for e in &files {
            let name = e.file_name().to_string_lossy().to_string();
            let path = e.path();
            let data = fs::read(&path)?;
            result.push(FileEntry { path, name, data });
        }

        Ok(result)
    }

    pub fn cleanup_files(&self, files: Vec<FileEntry>) -> anyhow::Result<()> {
        for file in files {
            let _ = fs::remove_file(file.path);
        }

        Ok(())
    }

    pub fn load_root_file(&self, id: Uuid) -> anyhow::Result<String> {
        let path = self.args.roots_storage_path.join(format!("{}.root", id));
        let s = fs::read_to_string(path)?;
        Ok(s.trim().to_string())
    }

    pub fn list_root_files(&self) -> anyhow::Result<Vec<Uuid>> {
        let entries: Vec<_> = fs::read_dir(self.args.roots_storage_path.clone())?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .collect();

        let mut result = Vec::new();

        for entry in entries {
            if let Some(filename) = entry.file_name().to_str()
                && let Some(id_str) = filename.strip_suffix(".root")
                && let Ok(id) = Uuid::parse_str(id_str)
            {
                result.push(id);
            }
        }

        Ok(result)
    }

    pub fn write_root_file(&self, id: Uuid, root_hex: &str) -> anyhow::Result<()> {
        fs::create_dir_all(self.args.roots_storage_path.clone())?;

        let path = self.args.roots_storage_path.join(format!("{}.root", id));

        fs::write(path, root_hex)?;

        Ok(())
    }
}
