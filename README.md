# 🦀 RvC (Remote Version Control)

A fast, lightweight reimplementation of **Git** written entirely in **Rust**.  
RvC showcases how version control systems manage commits, diffs, and file tracking under the hood while introducing a completely decentralized Peer-to-Peer (P2P) synchronization protocol.

---

## 🚀 Features

### Core Commands
| Command | Description |
|----------|--------------|
| `init` | Initialize a new repository |
| `add <file>` | Stage files for commit |
| `commit <message>` | Create a new commit |
| `log` | Display commit history |
| `status` | Show staged and unstaged changes |
| `diff` | Show line-by-line differences between working tree and index |

### P2P Sync layer
| Command | Description |
|----------|--------------|
| `start --port <PORT>` | Start a daemon node for the P2P network |
| `announce <repo>` | Announce your local repository to the DHT |
| `peers <repo>` | Find other peers hosting the repository |
| `sync <repo>` | Synchronize and merge commits with peers |

---

## 🏗 Architecture

RvC is designed with clean separation of concerns, divided into three primary layers:

### 1. Object Store & DAG Layer (Repo)
* **Immutable Objects**: Follows Git's content-addressable object model. Everything is addressed by the SHA-256 hash of its contents.
* **Nodes**: Includes `Blob` (file content), `Tree` (directory listings), and `Commit` (snapshots).
* **DAG History**: Commits form a Directed Acyclic Graph. Branch heads simply point to commit hashes. Updates are performed through fast-forwards or automatic merge commits.

### 2. Synchronization Layer
* **Protocol**: A highly optimized Request-Response protocol over `libp2p`.
* **Delta Resolution**: Nodes exchange `HEAD` hashes, traverse the DAG to identify missing commits and trees, and strictly fetch the required objects.
* **Conflict Resolution**: Divergent branches are resolved with auto-generated merge commits, unifying trees locally before updating the reference.

### 3. Network Layer (P2P)
* **Transport**: Built on `libp2p`, using TCP and Multiplexing for resilient peer-to-peer connections.
* **Discovery**: Uses **mDNS** for LAN auto-discovery and **Kademlia DHT** (Distributed Hash Table) for global peer routing and repository announcements.
* **Decentralized**: No central server is required. Repositories sync strictly peer-to-peer.

---

## ⚙️ Installation

```bash
# Clone repository
git clone https://github.com/mallaanirudh/RvC.git
cd RvC/minigit

# Build the binary
cargo build --release

# Run the CLI
./target/release/minigit
```

## 🧩 Example Workflow

```bash
# Initialize repository
minigit init

echo "Hello World" > story.txt

# Stage and Commit
minigit add story.txt
minigit commit "Add initial story"

# Display commit history
minigit log
```