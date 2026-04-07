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
    Node {
        port: Option<u16>,
    },
    Start {
        #[arg(long)]
        bootstrap: Option<String>,
        #[arg(long)]
        port: Option<u16>,
    },
    Announce {
        repo: String,
    },
    Peers {
        repo: String,
    },
    Sync {
        repo: String,
    },
    Checkout {
        hash: String,
    },
}