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

Minimal architecture for a P2P system with controlled writes.

* P2P network (no central relay)
* Single logical writer (authority)
* Deterministic state replication

---

## Layers

### 1. Network Layer (libp2p)

**Purpose:** transport only

Components:

* mDNS → peer discovery (LAN only)
* Connections → TCP/QUIC
* PubSub → broadcast messages
* Request–Response → direct sync

Rules:

* no state logic
* no validation
* no mutation

---

### 2. Repo Layer (State Store)

**Purpose:** local state + versioning

```rust
struct Repo<T> {
    data: T,
    version: u64
}
```

Rules:

* version increments strictly by 1
* accepts only `version == current + 1`
* rejects stale / duplicate / out-of-order updates
* no networking

API:

```rust
fn get() -> (T, u64)
fn apply(data: T, version: u64) -> bool
```

---

### 3. Sync Layer (Protocol)

**Purpose:** move state across nodes

#### PubSub (live updates)

Topic:

```
updates
```

Message:

```rust
struct UpdateMsg<T> {
    version: u64,
    data: T
}
```

---

#### Request–Response (recovery)

Protocol:

```
/sync/1.0.0
```

```rust
struct SyncRequest {
    from_version: u64
}

struct SyncResponse<T> {
    updates: Vec<UpdateMsg<T>>
}
```

---

### Sync Logic

```rust
if msg.version == repo.version + 1:
    apply
else if msg.version > repo.version + 1:
    request_sync()
else:
    ignore
```

---

## Authority Layer

**Purpose:** enforce single writer

Rules:

* exactly one node publishes updates
* all other nodes are read-only

### Authority Node

* updates repo
* increments version
* publishes UpdateMsg

### Follower Nodes

* subscribe to updates
* apply valid updates
* request missing data

---

## Data Flow

```
Authority:
    write → repo → version++ → publish

Follower:
    receive → validate → apply
           OR
           detect gap → sync request
```

---

## Constraints

* no multiple writers
* no DHT for syncing
* no full-state broadcast loops
* no high-frequency data syncing

---

## Known Limitations

* authority is a single point of failure
* no automatic failover
* mDNS limits network scope

---

## Summary

* Network = communication
* Sync = protocol
* Repo = state
* Authority = write control

Separation is mandatory. Mixing these will break consistency.
