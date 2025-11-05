use anyhow::Result;
use clap::Parser;
use minigit::cli::Cli;
use minigit::commands;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;

    match cli.command {
        commands::Commands::Init => commands::init::execute(&cwd)?,
        commands::Commands::Add { path } => commands::add::execute(&cwd, &path)?,
        commands::Commands::Commit { message } => commands::commit::execute(&cwd, &message)?,
        commands::Commands::Log => commands::log::execute(&cwd)?,
        commands::Commands::Status => commands::status::execute(&cwd)?, 
        commands::Commands::Diff => commands::diff::execute(&cwd)?, 
    }

    Ok(())
}