use anyhow::Result;
use clap::Parser;
use rvc::cli::Cli;
use rvc::commands;
use rvc::network;
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;

    match cli.command {
        commands::Commands::Init => commands::init::execute(&cwd)?,
        commands::Commands::Add { path } => commands::add::execute(&cwd, &path)?,
        commands::Commands::Commit { message } => commands::commit::execute(&cwd, &message)?,
        commands::Commands::Log => commands::log::execute(&cwd)?,
        commands::Commands::Status => commands::status::execute(&cwd)?, 
        commands::Commands::Diff => commands::diff::execute(&cwd)?, 
        commands::Commands::Node {port  } => { network::node::run_node(port, None).await.unwrap();},
        commands::Commands::Start { bootstrap, port } => { network::node::run_node(port, bootstrap).await.unwrap();},
        commands::Commands::Announce { repo } => { network::node::announce_cmd(&cwd, &repo).await.unwrap();},
        commands::Commands::Peers { repo } => { network::node::peers_cmd(&cwd, &repo).await.unwrap();},
        commands::Commands::Sync { repo } => { network::node::sync_cmd(&cwd, &repo).await.unwrap();},
    }
    Ok(())
}