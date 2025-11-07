use crate::{
    FileManager,
    commands::{Command, helpers::get_path_from_str},
    file_manager::FileManagerArgs,
};
use async_trait::async_trait;
use clap::{Arg, ArgAction, ArgMatches};
use std::path::PathBuf;

struct ListUploadIdsCommandArgs {
    files_directory: PathBuf,
    roots_store_directory: PathBuf,
}

impl From<&ArgMatches> for ListUploadIdsCommandArgs {
    fn from(args: &ArgMatches) -> Self {
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
            files_directory,
            roots_store_directory,
        }
    }
}

impl From<&ListUploadIdsCommandArgs> for FileManagerArgs {
    fn from(val: &ListUploadIdsCommandArgs) -> Self {
        FileManagerArgs {
            files_storage_path: val.files_directory.clone(),
            roots_storage_path: val.roots_store_directory.clone(),
        }
    }
}

pub struct ListUploadIdsCommand;

impl ListUploadIdsCommand {
    async fn list_upload_ids(&self, file_manager: FileManager) -> anyhow::Result<()> {
        let upload_ids = file_manager.list_root_files()?;

        if upload_ids.is_empty() {
            println!("No upload IDs found.");
        } else {
            println!("Upload IDs:");
            for id in upload_ids {
                println!("- {}", id);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Command for ListUploadIdsCommand {
    fn create(&self) -> clap::Command {
        clap::Command::new("list-upload-ids")
            .about("List all upload IDs")
            .long_flag("list-upload-ids")
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
    }

    fn name(&self) -> String {
        "list-upload-ids".to_owned()
    }

    async fn execute(&self, args: &ArgMatches) {
        let commands_args: ListUploadIdsCommandArgs = args.into();
        let file_manager_args: FileManagerArgs = (&commands_args).into();

        let file_manager = FileManager::new(file_manager_args);

        self.list_upload_ids(file_manager)
            .await
            .expect("Failed to list upload IDs");
    }
}
