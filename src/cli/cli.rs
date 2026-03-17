use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rvc")]
#[command(about = "A p2p dectralized version control", long_about = None)]
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
    Node,
}