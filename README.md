# 🦀 RvC: Remote Version Control

[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)]()

**RvC (Remote Version Control)** is a decentralized, peer-to-peer (P2P) version control system designed for serverless collaboration. Built entirely in Rust, it combines a Git-compatible content-addressable object model with a modern P2P networking stack using `libp2p`.

Unlike traditional version control systems that rely on central servers (like GitHub or GitLab), RvC allows nodes to discover each other on a local network or globally via a Distributed Hash Table (DHT), synchronized divergent histories, and merge changes directly between peers.

---

## ✨ Key Features

- **Decentralized Synchronization**: Sync repositories directly peer-to-peer without a central authority.
- **P2P Discovery**: 
  - **mDNS**: Automatic discovery of peers on the same local network.
  - **Kademlia DHT**: Global routing and repository announcements for wide-area sync.
- **Iterative Object Fetching**: Efficiently transfers only the missing commits, trees, and blobs using a specialized request-response protocol.
- **Automated Conflict Resolution**: Per-file union merging for non-conflicting changes with automated checkout.
- **Content-Addressable Storage**: Uses BLAKE3 hashing for immutable object integrity.
- **Familiar Workspace**: Git-like CLI commands for initialization, staging, and committing.

---

## 🏗 Architecture

RvC is built on a three-layer architecture:

1.  **Storage Layer**: Manages the `.rvc` directory, storing immutable objects (Blobs, Trees, Commits) and references (HEAD).
2.  **Sync Layer**: Orchestrates the graph traversal algorithm to identify missing objects between two divergent commit histories.
3.  **Network Layer**: Leverages `libp2p` with Noise encryption and Yamux multiplexing to provide secure, resilient streams between peers.

---

## 🚀 Getting Started

### Prerequisites

- **Rust Toolchain**: [Install Rust](https://rustup.rs/) (1.70 or higher recommended).
- **Git** (Optional, for cloning the source).

### Installation (Building from Source)

The recommended way to install RvC is to build it from the source:

```bash
# Clone the repository
git clone https://github.com/mallaanirudh/RvC.git
cd RvC/minigit

# Build and install to your local cargo binary path
cargo install --path .
```

Alternatively, you can build the binary and move it manually:
```bash
cargo build --release
# Binary is located at ./target/release/rvc
```

### Installation (Downloading Release)

For users who prefer pre-compiled binaries:
1. Navigate to the [Releases](https://github.com/mallaanirudh/RvC/releases) page.
2. Download the appropriate binary for your operating system (e.g., `rvc-windows.exe`, `rvc-linux`).
3. Place the binary in a directory included in your system's `PATH`.
4. Verify installation:
   ```bash
   rvc --version
   ```

---

## 📖 Command Reference

### Core Version Control

| Command | Usage | Description |
| :--- | :--- | :--- |
| `init` | `rvc init` | Initialize a new RvC repository in the current directory. |
| `add` | `rvc add <file>` | Stage a file or directory for the next commit. |
| `commit` | `rvc commit "<msg>"` | Record the staged changes into a new commit. |
| `status` | `rvc status` | Show the status of files in the workspace (staged/unstaged). |
| `log` | `rvc log` | Display the commit history of the current branch. |
| `diff` | `rvc diff` | Show line-by-line changes between the workspace and the last commit. |
| `checkout` | `rvc checkout <hash>` | Restore the workspace files to a specific commit state. |

### P2P Networking & Sync

| Command | Usage | Description |
| :--- | :--- | :--- |
| `start` | `rvc start [--port <p>]` | Start a background node (daemon) for network discovery. |
| `announce` | `rvc announce <project>` | Announce your local project to the DHT for others to find. |
| `peers` | `rvc peers <project>` | List all discovered peers for a specific project. |
| `sync` | `rvc sync <project>` | Discover peers and pull/merge changes from them. |

---

## 🛠 Advanced Usage: Peer-to-Peer Sync

To synchronize changes between two machines:

1.  **Machine A** (The host):
    ```bash
    rvc start --port 4001
    rvc announce my-awesome-project
    ```
2.  **Machine B** (The client):
    ```bash
    rvc sync my-awesome-project
    ```
RvC will automatically find Machine A, identify any missing commits, fetch them, merge them into your local history, and update your working directory.

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1.  Fork the Project.
2.  Create your Feature Branch (`git checkout -b feature`).
3.  Commit your Changes (`git commit -m 'Add feature'`).
4.  Push to the Branch (`git push origin feature`).
5.  Open a Pull Request.

---

