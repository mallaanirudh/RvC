# upload-1762361776495.pdf
## Metadata
- PDFFormatVersion=1.4
- IsLinearized=false
- IsAcroFormPresent=false
- IsXFAPresent=false
- IsCollectionPresent=false
- IsSignaturesPresent=false
- Author=(anonymous)
- CreationDate=D:20251105165427+00'00'
- Creator=(unspecified)
- ModDate=D:20251105165427+00'00'
- Producer=ReportLab PDF Library - www.reportlab.com
- Subject=(unspecified)
- Title=(anonymous)
- Trapped=[object Object]
## Contents
### Page 1
#   n   RvC (MiniGit in Rust) A lightweight educational reimplementation of Git written in Rust. RvC demonstrates how version control systems manage commits, diffs, and file tracking internally while keeping the c --- ##   n   Features ### Core Commands | Command | Description | |----------|--------------| | `init` | Initialize a new repository | | `add <file>` | Stage files for commit | | `commit -m <message>` | Create a new commit | | `log` | Display commit history | | `status` | Show staged and unstaged changes | | `diff` | Show line-by-line differences between working tree and index | --- ##   n   Example Workflow ```bash # Initialize minigit init # Create a file echo "Hello World" > story.txt # Check status (shows untracked files) minigit status # Stage files minigit add story.txt # Check status again (shows staged files) minigit status # Commit minigit commit -m "Add initial story" # Modify file echo "New chapter" >> story.txt # View changes before committing minigit diff # View commit history minigit log ``` --- ##   n   Project Structure ```



### Page 2
minigit/  nnn   src/  n   nnn   commands/  n   n   nnn   init.rs  n   n   nnn   add.rs  n   n   nnn   commit.rs  n   n   nnn   log.rs  n   n   nnn   status.rs  n   n   nnn   diff.rs  n   nnn   cli/  n   nnn   core/  n   nnn   index/  nnn   tests/  nnn   .gitignore  nnn   README.md ``` --- ##   nn   Installation ```bash # Clone repository git clone https://github.com/mallaanirudh/RvC.git cd RvC # Build the binary cargo build --release # Run CLI ./target/release/minigit ``` --- ##   n   Concepts Behind the Project This project demonstrates: - Git’s content-addressable storage model - File staging/indexing logic - Commit graph traversal - Diff and status generation between working tree, index, and HEAD --- ##   n   Testing ```bash cargo test ``` Unit tests cover core modules such as index, commit store, and diff engine. --- ##   n   Future Plans  - Implement branching and merging



### Page 3
- Add file rename detection - Build a simple TUI for visualization - Integrate RvC with other Rust-based DevOps tools --- ##   nnn   Author **Mallaa Anirudh** Rust developer and systems programming enthusiast [GitHub Profile](https://github.com/mallaanirudh) --- ##   n   License MIT License © 2025 Mallaa Anirudh

