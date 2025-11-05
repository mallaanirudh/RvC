
## 🚀 Features

### Core Commands
- `init` - Initialize a new repository
- `add <file>` - Stage files for commit  
- `commit -m <message>` - Create new commits
- `log` - Display commit history
- `status` - Show staged/unstaged changes ← **NEW**
- `diff` - Show line-by-line changes ← **NEW**

## Example Workflow

```bash
# Initialize
minigit init

# Create files
echo "Hello World" > story.txt

# Check status (shows untracked files)
minigit status

# Stage files
minigit add story.txt

# Check status (shows staged changes)  
minigit status

# Commit
minigit commit -m "Add initial story"

# Make changes
echo "New chapter" >> story.txt

# See what changed
minigit diff

# View history
minigit log
text

## 4. Final project structure check:

Your project should now have:
minigit/
├── src/
│ ├── commands/
│ │ ├── init.rs
│ │ ├── add.rs
│ │ ├── commit.rs
│ │ ├── log.rs
│ │ ├── status.rs ← NEW
│ │ └── diff.rs ← NEW
│ ├── cli/
│ ├── core/
│ └── index/
├── tests/ ← NEW
├── .gitignore
└── README.md

text

## 5. Push everything to GitHub:

```cmd
git add .
git commit -m "docs: Update README with new features"
git push
