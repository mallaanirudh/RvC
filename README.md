# 🦀 RvC (Rust Version Control)

A  reimplementation of **Git**, written entirely in **Rust**.  
RvC showcases how version control systems manage commits, diffs, and file tracking under the hood — in a compact, understandable codebase.

---

## 🚀 Features

### Core Commands
| Command | Description |
|----------|--------------|
| `init` | Initialize a new repository |
| `add <file>` | Stage files for commit |
| `commit -m <message>` | Create a new commit |
| `log` | Display commit history |
| `status` | Show staged and unstaged changes |
| `diff` | Show line-by-line differences between working tree and index |

---

## 🧩 Example Workflow

```bash
# Initialize repository
minigit init

# Create a file
echo "Hello World" > story.txt

# Check status (shows untracked files)
rvc status

# Stage files
rvc add story.txt

# Verify staged files
rvc status

# Commit
rvc commit -m "Add initial story"

# Modify file
echo "New chapter" >> story.txt

# View file changes
rvc diff

# Display commit history
rvc log

⚙️ Installation
# Clone repository
git clone https://github.com/mallaanirudh/RvC.git
cd RvC/minigit

# Build the binary
cargo build --release

# Run the CLI (on Windows)
./target/release/minigit.exe

# Or during development
cargo run -- [command]
