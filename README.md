MiniGit - A Minimal Git Implementation in Rust
A educational implementation of core Git functionality in Rust, designed to demonstrate data structures and algorithms used in version control systems.

🚀 Features Implemented
Core VCS Functionality
Object Database: Content-addressable storage for blobs, trees, and commits

Hashing: Blake3 for content-based addressing (like Git's SHA-1)

Staging Area: JSON-backed index for tracking changes

Commit History: Directed acyclic graph (DAG) of commits with parent pointers

Commands
init - Initialize a new repository

add - Stage files for commit

commit - Create new commits with author, message, and timestamp

log - Display commit history
