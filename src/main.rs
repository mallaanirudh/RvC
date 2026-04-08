use anyhow::Result;
use clap::Parser;
use rvc::cli::Cli;
use rvc::commands;
use rvc::network;
#[tokio::main]
async fn main() -> Result<()> {
    println!("RVC_VERSION: SYNC_STABLE_V3");
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;

    match cli.command {
        commands::Commands::Init => commands::init::execute(&cwd)?,
        commands::Commands::Add { path } => commands::add::execute(&cwd, &path)?,
        commands::Commands::Commit { message } => commands::commit::execute(&cwd, &message)?,
        commands::Commands::Log => commands::log::execute(&cwd)?,
        commands::Commands::Status => commands::status::execute(&cwd)?, 
        commands::Commands::Diff => commands::diff::execute(&cwd)?, 
        commands::Commands::Node { port } => { 
            if let Err(e) = network::node::run_node(port, None).await {
                eprintln!("Node error: {}", e);
                std::process::exit(1);
            }
        },
        commands::Commands::Start { bootstrap, port } => { 
            if let Err(e) = network::node::run_node(port, bootstrap).await {
                eprintln!("Start error: {}", e);
                std::process::exit(1);
            }
        },
        commands::Commands::Announce { repo, port } => { 
            if let Err(e) = network::node::announce_cmd(&cwd, &repo, port).await {
                eprintln!("Announce error: {}", e);
                std::process::exit(1);
            }
        },
        commands::Commands::Peers { repo } => { 
            if let Err(e) = network::node::peers_cmd(&cwd, &repo).await {
                eprintln!("Peers error: {}", e);
                std::process::exit(1);
            }
        },
        commands::Commands::Sync { repo, port } => { 
            if let Err(e) = network::node::sync_cmd(&cwd, &repo, port).await {
                eprintln!("Sync error: {}", e);
                std::process::exit(1);
            }
        },
        commands::Commands::Checkout { hash } => commands::checkout::execute(&cwd, &hash)?,
    }
    Ok(())
}