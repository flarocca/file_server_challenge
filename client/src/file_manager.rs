// The functionality of this FileManager, suchas as writting or loading files, was delegated to
// ChatGPT, I then took the functions and refactored them as I liked.
// One of those big refactors was to replace `fs` with `tokio::fs` to make it async
use std::path::PathBuf;
use tokio::fs;
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

    pub async fn load_files(&self) -> anyhow::Result<Vec<FileEntry>> {
        let mut rd = fs::read_dir(&self.args.files_storage_path).await?;
        let mut entries = Vec::new();

        while let Some(entry) = rd.next_entry().await? {
            let meta = entry.metadata().await?;
            if meta.is_file() {
                entries.push(entry);
            }
        }

        if entries.is_empty() {
            return Ok(Vec::new());
        }

        let mut result = Vec::with_capacity(entries.len());
        for e in entries {
            let name = e.file_name().to_string_lossy().to_string();
            let path = e.path();
            let data = fs::read(&path).await?;
            result.push(FileEntry { path, name, data });
        }

        Ok(result)
    }

    pub async fn cleanup_files(&self, files: Vec<FileEntry>) -> anyhow::Result<()> {
        for file in files {
            let _ = fs::remove_file(file.path).await;
        }

        Ok(())
    }

    pub async fn load_root_file(&self, id: Uuid) -> anyhow::Result<String> {
        let path = self.args.roots_storage_path.join(format!("{}.root", id));

        let s = fs::read_to_string(path).await?;

        Ok(s.trim().to_string())
    }

    pub async fn list_root_files(&self) -> anyhow::Result<Vec<Uuid>> {
        let mut rd = fs::read_dir(&self.args.roots_storage_path).await?;
        let mut result = Vec::new();

        while let Some(entry) = rd.next_entry().await? {
            let meta = entry.metadata().await?;
            if !meta.is_file() {
                continue;
            }

            if let Some(filename) = entry.file_name().to_str()
                && let Some(id_str) = filename.strip_suffix(".root")
                && let Ok(id) = Uuid::parse_str(id_str)
            {
                result.push(id);
            }
        }

        Ok(result)
    }

    pub async fn write_root_file(&self, id: Uuid, root_hex: &str) -> anyhow::Result<()> {
        fs::create_dir_all(&self.args.roots_storage_path).await?;

        let path = self.args.roots_storage_path.join(format!("{}.root", id));

        fs::write(path, root_hex).await?;

        Ok(())
    }
}
