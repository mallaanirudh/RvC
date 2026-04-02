# 🦀 RvC (Remote Version Control)

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
# RvC Networking, Sync, and Repo Architecture (Authority-Based)

## Overview

Layers
1. Network Layer (libp2p)

Purpose: transport only

Components:

mDNS → peer discovery (LAN; replaceable with DHT later)
Connections → TCP/QUIC
PubSub → lightweight announcements
Request–Response → object transfer

Rules:

no state logic
no validation
no mutation
2. Repo Layer (Object Store + DAG)

Purpose: local state + history

Core Objects
type Hash = [u8; 32];

struct Blob {
    data: Vec<u8>
}

struct Tree {
    entries: Vec<(String, Hash)>
}

struct Commit {
    tree: Hash,
    parents: Vec<Hash>,
    author: String,
    message: String,
    timestamp: u64
}
Storage
trait ObjectStore {
    fn get(&self, hash: &Hash) -> Option<Vec<u8>>;
    fn put(&mut self, data: Vec<u8>) -> Hash;
    fn has(&self, hash: &Hash) -> bool;
}

Rules:

objects are immutable
hash = content identity
no global version counter
References (Branches)
struct Refs {
    heads: HashMap<String, Hash>
}

Rules:

branch = pointer to commit
multiple branches allowed
updates must be fast-forward or merged
Repo API
fn commit(tree: Hash, parents: Vec<Hash>, msg: String) -> Hash;

fn get_commit(hash: Hash) -> Commit;

fn update_ref(name: &str, new_head: Hash);

fn get_ref(name: &str) -> Option<Hash>;
Key Property
History forms a DAG, not a linear chain
Concurrent commits are valid and expected
3. Sync Layer (Protocol)

Purpose: exchange objects and refs across nodes

Request–Response Protocol

Protocol:

/sync/2.0.0
Messages
enum SyncRequest {
    GetRefs,
    GetObjects(Vec<Hash>)
}

enum SyncResponse {
    Refs(HashMap<String, Hash>),
    Objects(Vec<(Hash, Vec<u8>)>)
}
PubSub (optional)

Topic:

refs-update

Message:

struct RefUpdate {
    branch: String,
    head: Hash
}

Purpose:

notify peers of new commits
trigger pull-based sync
Sync Logic
On Peer Connection
1. exchange refs
2. compare branch heads
3. find missing commits via DAG traversal
4. request missing objects
5. store objects locally
6. update refs (fast-forward or merge)
Missing Object Discovery
if !local.has(hash):
    request object
    traverse parents
Update Rules
if new_head is descendant of local_head:
    fast-forward
else:
    create merge commit
Data Flow
Node A:
    commit → store objects → update ref → announce

Node B:
    receive ref update → compare → fetch missing objects
           → validate → store → update ref