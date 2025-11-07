use anyhow::Result;
use file_server_client::get_commands;
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Layer, Registry, fmt, layer::SubscriberExt};

fn init_tracing() {
    LogTracer::init().expect("Failed to set logger");
    let fmt_layer = fmt::layer()
        .pretty()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .json()
        .boxed();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let subscriber = Registry::default().with(filter_layer).with(fmt_layer);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let commands = get_commands();

    let mut clap_commands = clap::Command::new("file-server-client")
        .version("0.1.0")
        .subcommand_required(true)
        .arg_required_else_help(true);

    for command in commands.values() {
        clap_commands = clap_commands.subcommand(command.create());
    }

    let matches = clap_commands.get_matches();
    match matches.subcommand() {
        Some(subcommand) => {
            let (subcommand_name, subcommand_args) = subcommand;
            let command = commands.get(subcommand_name).unwrap();
            command.execute(subcommand_args).await;
        }
        _ => {
            println!("No subcommand provided");
        }
    }

    Ok(())
}
