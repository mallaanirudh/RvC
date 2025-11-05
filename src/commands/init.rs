use crate::core;
use anyhow::Result;
use std::path::Path;

// Remove: pub struct InitCommand;

pub fn execute(repo: &Path) -> Result<()> {
    core::init(repo)?;
    println!("initialized .minigit");
    Ok(())
}