use crate::core::{FsObjectStore, Oid};
use anyhow::Result;
use std::collections::VecDeque;
use std::fs;
use std::path::Path;

// Remove: pub struct LogCommand;

pub fn execute(repo: &Path) -> Result<()> {
    let store = FsObjectStore::new(repo);
    let logs = get_logs(repo, &store)?;
    
    for entry in logs {
        println!("{}", entry);
    }
    
    Ok(())
}

fn get_logs(repo: &Path, store: &FsObjectStore) -> Result<Vec<String>> {
    let headp = crate::core::head_file(repo);
    if !headp.exists() {
        return Ok(vec![]);
    }

    let head = fs::read_to_string(headp)?.trim().to_string();
    if head.is_empty() {
        return Ok(vec![]);
    }

    let bytes = hex::decode(&head)?;
    let oid = Oid::from(&bytes[..]);
    let mut out = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(oid);

    while let Some(cur) = queue.pop_front() {
        if let Some(crate::core::Object::Commit(c)) = store.get(&cur)? {
            out.push(format!(
                "commit {}\nAuthor: {}\nDate: {}\n\n    {}\n",
                cur, c.author, c.timestamp, c.message
            ));

            for parent in c.parents.iter() {
                let pb = hex::decode(parent)?;
                queue.push_back(Oid::from(&pb[..]));
            }
        }
    }

    Ok(out)
}