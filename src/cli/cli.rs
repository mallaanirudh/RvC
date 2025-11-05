use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rvc")]
#[command(about = "Minimal Git-like VCS for DSA demonstration", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    Add {
        path: String,
    },
    Commit {
        message: String,
    },
    Log,
    Status,
    Diff,
}