pub mod add;
pub mod commit;
pub mod init;
pub mod log;

// Re-export the Commands enum from cli module
pub use crate::cli::cli::Commands;