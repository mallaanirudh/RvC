pub mod add;
pub mod commit;
pub mod init;
pub mod log;
pub mod status;
pub mod diff;

// Re-export the Commands enum from cli module
pub use crate::cli::cli::Commands;