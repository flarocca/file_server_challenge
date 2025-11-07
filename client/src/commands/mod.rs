// This design for commands allows for extensibility and easy addition of new commands in the
// future. This is a custom implementation I had already written in the past ([see here](https://github.com/flarocca/rust_revm_simulations/blob/main/src/commands/mod.rs))
// It could be argued that using the typed version of commands would be better, but I think it is
// less flexible and more coupled.
mod helpers;
mod list_upload_ids;
mod upload_files;
mod verify_file;

pub use list_upload_ids::ListUploadIdsCommand;
pub use upload_files::UploadFilesCommand;
pub use verify_file::VerifyFileCommand;

use async_trait::async_trait;
use clap::ArgMatches;
use std::collections::HashMap;

#[async_trait]
pub trait Command {
    async fn execute(&self, args: &ArgMatches);

    fn create(&self) -> clap::Command;

    fn name(&self) -> String;
}

pub fn get_commands() -> HashMap<String, Box<dyn Command>> {
    let mut result = HashMap::new();

    let commands: Vec<Box<dyn Command>> = vec![
        Box::new(UploadFilesCommand),
        Box::new(VerifyFileCommand),
        Box::new(ListUploadIdsCommand),
    ];

    for command in commands {
        result.insert(command.name(), command);
    }

    result
}
