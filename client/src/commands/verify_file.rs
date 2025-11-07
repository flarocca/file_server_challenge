use std::path::PathBuf;

use async_trait::async_trait;
use clap::{Arg, ArgAction, ArgMatches, value_parser};
use file_server_library::{CustomMerkleTree, models::Hash32};
use reqwest::Url;
use uuid::Uuid;

use crate::{
    ApiClient, ApiClientArgs, FileManager,
    commands::{Command, helpers::get_path_from_str},
    file_manager::FileManagerArgs,
};

struct VerifyFilesCommandArgs {
    api_key: String,
    api_secret: String,
    base_url: String,
    files_directory: PathBuf,
    roots_store_directory: PathBuf,
    index: usize,
    id: Uuid,
}

impl From<&ArgMatches> for VerifyFilesCommandArgs {
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

        let id: Uuid = args
            .get_one::<String>("id")
            .expect("Upload ID is required")
            .parse()
            .expect("Failed to parse Upload ID");

        let index = args.get_one::<usize>("index").expect("Index is required");

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
            index: *index,
            id,
        }
    }
}

impl From<&VerifyFilesCommandArgs> for ApiClientArgs {
    fn from(val: &VerifyFilesCommandArgs) -> Self {
        ApiClientArgs {
            api_key: val.api_key.clone(),
            api_secret: val.api_secret.clone(),
            base_url: Url::parse(&val.base_url).expect("Failed to parse base URL"),
            correlation_id: Uuid::new_v4(),
        }
    }
}

impl From<&VerifyFilesCommandArgs> for FileManagerArgs {
    fn from(val: &VerifyFilesCommandArgs) -> Self {
        FileManagerArgs {
            files_storage_path: val.files_directory.clone(),
            roots_storage_path: val.roots_store_directory.clone(),
        }
    }
}

pub struct VerifyFileCommand;

impl VerifyFileCommand {
    // As upload file, this function was created by ChatGPT based on a detailed prompt
    // and refactored by me.
    async fn verify_file(
        &self,
        api_client: ApiClient,
        file_manager: FileManager,
        args: VerifyFilesCommandArgs,
    ) -> anyhow::Result<()> {
        let root_hex = file_manager.load_root_file(args.id)?;
        let root = Hash32::from_hex(&root_hex).map_err(|e| anyhow::anyhow!(e))?;

        let file_bytes = api_client.download_file(args.id, args.index).await?;

        let leaf = Hash32::hash(&file_bytes);

        let proof = api_client.get_proof(args.id, args.index).await?;
        let ok = CustomMerkleTree::verify(&leaf, &proof, &root);

        if ok {
            println!(
                "File verification succeeded for id={}, index={}",
                args.id, args.index
            );
        } else {
            eprintln!(
                "File verification failed for id={}, index={}",
                args.id, args.index
            );
        }

        Ok(())
    }
}

#[async_trait]
impl Command for VerifyFileCommand {
    fn create(&self) -> clap::Command {
        clap::Command::new("verify-file")
            .about("This command verifies that a file for a given index is valid.")
            .long_flag("verify-file")
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
            .arg(
                Arg::new("id")
                    .long("id")
                    .short('i')
                    .action(ArgAction::Set)
                    .help("Upload ID to verify"),
            )
            .arg(
                Arg::new("index")
                    .long("index")
                    .short('x')
                    .value_parser(value_parser!(usize))
                    .action(ArgAction::Set)
                    .help("Index of the file to verify"),
            )
            .arg_required_else_help(true)
    }

    fn name(&self) -> String {
        "verify-file".to_owned()
    }

    async fn execute(&self, args: &ArgMatches) {
        let commands_args: VerifyFilesCommandArgs = args.into();

        let api_args: ApiClientArgs = (&commands_args).into();
        let file_manager_args: FileManagerArgs = (&commands_args).into();

        let api_cli = ApiClient::new(api_args).expect("Failed to create API client");
        let file_manager = FileManager::new(file_manager_args);

        self.verify_file(api_cli, file_manager, commands_args)
            .await
            .expect("Failed to verify file");
    }
}
