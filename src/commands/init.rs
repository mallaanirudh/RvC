use crate::core;
use anyhow::Result;
use std::path::Path;
pub fn execute(repo: &Path) -> Result<()> {
    core::init(repo)?;
    println!("initialized .minigit");
    Ok(())
}