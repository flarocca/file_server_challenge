use async_trait::async_trait;
use clap::{Arg, ArgAction, ArgMatches};
use file_server_library::{CustomMerkleTree, models::Hash32};
use reqwest::Url;
use std::path::PathBuf;
use uuid::Uuid;

use crate::{
    ApiClient, ApiClientArgs, FileManager,
    commands::{Command, helpers::get_path_from_str},
    file_manager::FileManagerArgs,
};

struct UploadFilesCommandArgs {
    api_key: String,
    api_secret: String,
    base_url: String,
    files_directory: PathBuf,
    roots_store_directory: PathBuf,
}

impl From<&ArgMatches> for UploadFilesCommandArgs {
    fn from(args: &ArgMatches) -> Self {
        let api_key = args
            .get_one::<String>("api-key")
            .expect("API-KEY is required")
            .to_owned();

        let api_secret = args
            .get_one::<String>("api-secret")
            .expect("API-SECRET is required")
            .to_owned();

        let base_url = args
            .get_one::<String>("base-url")
            .expect("Base URL is required")
            .to_owned();

        let files_directory = args
            .get_one::<String>("files-directory")
            .expect("File directory is required");
        let files_directory =
            get_path_from_str(files_directory).expect("Failed to parse files directory");

        let roots_store_directory = args
            .get_one::<String>("roots-store-directory")
            .expect("File directory is required");
        let roots_store_directory = get_path_from_str(roots_store_directory)
            .expect("Failed to parse roots store directory");

        Self {
            api_key,
            api_secret,
            base_url,
            files_directory,
            roots_store_directory,
        }
    }
}

impl From<&UploadFilesCommandArgs> for ApiClientArgs {
    fn from(val: &UploadFilesCommandArgs) -> Self {
        ApiClientArgs {
            api_key: val.api_key.clone(),
            api_secret: val.api_secret.clone(),
            base_url: Url::parse(&val.base_url).expect("Failed to parse base URL"),
            correlation_id: Uuid::new_v4(),
        }
    }
}

impl From<&UploadFilesCommandArgs> for FileManagerArgs {
    fn from(val: &UploadFilesCommandArgs) -> Self {
        FileManagerArgs {
            files_storage_path: val.files_directory.clone(),
            roots_storage_path: val.roots_store_directory.clone(),
        }
    }
}

pub struct UploadFilesCommand;

impl UploadFilesCommand {
    // This function was delegated to ChatGPT  based on the OpenAPI Json
    // and a detailed description of how the flow worked.
    // Then I refactored it as per my coding style.
    pub async fn upload_and_persist(
        &self,
        file_manager: FileManager,
        api_client: ApiClient,
    ) -> anyhow::Result<()> {
        let file_entries = file_manager.load_files().await?;
        if file_entries.is_empty() {
            anyhow::bail!("directory has no files");
        }
        let mut leaves = Vec::with_capacity(file_entries.len());
        let mut filenames = Vec::with_capacity(file_entries.len());

        for entry in &file_entries {
            leaves.push(Hash32::hash(&entry.data));
            filenames.push((entry.name.to_owned(), entry.data.clone()));
        }

        let custom_tree = CustomMerkleTree::new(leaves);
        let local_root = custom_tree.root();
        let root_hex = local_root.to_hex();

        let id = api_client.initiate().await?;

        for (idx, (name, data)) in filenames.into_iter().enumerate() {
            let _ = api_client.upload_file(id, &name, idx, data).await?;
        }

        let server_root = api_client.complete(id).await?;
        if server_root.to_ascii_lowercase() != root_hex {
            anyhow::bail!(
                "server root mismatch (server {}, local {})",
                server_root,
                root_hex
            );
        }

        file_manager.write_root_file(id, &root_hex).await?;
        file_manager.cleanup_files(file_entries).await?;

        println!("upload complete. id={}, root={}", id, root_hex);

        Ok(())
    }
}

#[async_trait]
impl Command for UploadFilesCommand {
    fn create(&self) -> clap::Command {
        // TODO: These set of command arguments are mostly common to all commands.
        // A very nice to have would be an abstraction that allows each command
        // to declare only the ones that exclusive for the command in question
        clap::Command::new("upload-files")
            .about("This command initiates, uploads and completes an upload flow.")
            .long_flag("upload-files")
            .arg(
                Arg::new("api-key")
                    .long("api-key")
                    .short('k')
                    .required(true)
                    .action(ArgAction::Set)
                    .help("API Key for authentication"),
            )
            .arg(
                Arg::new("api-secret")
                    .long("api-secret")
                    .short('s')
                    .required(true)
                    .action(ArgAction::Set)
                    .help("API Secret for authentication"),
            )
            .arg(
                Arg::new("base-url")
                    .long("base-url")
                    .short('u')
                    .default_value("http://localhost:8080")
                    .action(ArgAction::Set)
                    .help("API Secret for authentication"),
            )
            .arg(
                Arg::new("files-directory")
                    .long("files-directory")
                    .short('f')
                    .default_value("~/files")
                    .action(ArgAction::Set)
                    .help("Local directory containing files to upload"),
            )
            .arg(
                Arg::new("roots-store-directory")
                    .long("roots-store-directory")
                    .short('r')
                    .default_value("~/roots")
                    .action(ArgAction::Set)
                    .help("Local directory to persist upload roots"),
            )
            .arg_required_else_help(true)
    }

    fn name(&self) -> String {
        "upload-files".to_owned()
    }

    async fn execute(&self, args: &ArgMatches) {
        let commands_args: UploadFilesCommandArgs = args.into();

        let api_args: ApiClientArgs = (&commands_args).into();
        let file_manager_args: FileManagerArgs = (&commands_args).into();

        let api_cli = ApiClient::new(api_args).expect("Failed to create API client");
        let file_manager = FileManager::new(file_manager_args);

        self.upload_and_persist(file_manager, api_cli)
            .await
            .expect("Failed to upload files");
    }
}
