# Demo Project for yeth

This demo project demonstrates various features of `yeth`:

- Application dependencies
- File and directory dependencies
- Exclusion patterns
- Circular dependency detection
- Hash calculation with dependencies

## Structure

```
demo/
├── shared/              # Base library (no dependencies)
│   ├── yeth.toml
│   ├── utils.js
│   ├── constants.js
│   └── README.md
├── common/              # Utilities (depends on: shared)
│   ├── yeth.toml
│   └── lib.rs
├── config/              # Configuration files (no yeth.toml - used as path dependency)
│   ├── database.json
│   └── api.yaml
├── backend/             # Backend service (depends on: common, shared, config files)
│   ├── yeth.toml
│   ├── main.rs
│   ├── routes.rs
│   ├── handlers.rs
│   └── middleware.rs
├── frontend/            # Frontend app (depends on: backend, shared, config dir)
│   ├── yeth.toml
│   ├── package.json
│   ├── index.html
│   └── src/
│       ├── main.js
│       └── components/
│           └── App.js
└── admin/               # Admin panel (depends on: frontend, backend, common, shared/README.md)
    ├── yeth.toml
    ├── index.js
    └── README.md
```

## Dependency Graph

```
shared
  └─ (no dependencies)

common
  └─ shared (app)

backend
  ├─ common (app)
  ├─ shared (app)
  ├─ ../config/database.json (file)
  └─ ../config/api.yaml (file)

frontend
  ├─ backend (app)
  ├─ shared (app)
  └─ ../config (dir)

admin
  ├─ frontend (app)
  ├─ backend (app)
  ├─ common (app)
  └─ ../shared (dir)
  Note: excludes ../shared/README.md from hash
```

## Testing Commands

From the project root:

```bash
# Build yeth
cargo build --release

# Show all application hashes
./target/release/yeth --root demo

# Show dependency graph
./target/release/yeth --root demo --show-graph

# Show specific app hash
./target/release/yeth --root demo --app frontend

# Get only hash (for scripts)
./target/release/yeth --root demo --app backend --hash-only

# Show with statistics
./target/release/yeth --root demo --verbose

# Save hashes to yeth.version files
./target/release/yeth --root demo --write-versions
```

## Testing Scenarios

### 1. Basic hash calculation
```bash
./target/release/yeth --root demo
```
Should output hashes for all 5 applications (shared, common, backend, frontend, admin).

### 2. Dependency changes propagation
```bash
# Get initial hash for frontend
./target/release/yeth --root demo --app frontend --hash-only

# Modify shared library
echo "// comment" >> demo/shared/utils.js

# Check frontend hash again - should be different!
./target/release/yeth --root demo --app frontend --hash-only
```

### 3. Path dependency changes
```bash
# Get backend hash
./target/release/yeth --root demo --app backend --hash-only

# Modify config file
echo '  "extra": true' >> demo/config/database.json

# Check backend hash - should change
./target/release/yeth --root demo --app backend --hash-only
```

### 4. Exclusion patterns
```bash
# Create excluded directory
mkdir -p demo/frontend/node_modules/somepackage
echo "content" > demo/frontend/node_modules/somepackage/index.js

# Hash should NOT change (node_modules is excluded)
./target/release/yeth --root demo --app frontend --hash-only
```

### 5. Path-based exclusions in admin
```bash
# Get admin hash
./target/release/yeth --root demo --app admin --hash-only

# Modify shared/README.md
echo "update" >> demo/shared/README.md

# Admin hash should NOT change (../shared/README.md is excluded)
./target/release/yeth --root demo --app admin --hash-only

# But shared hash SHOULD change
./target/release/yeth --root demo --app shared --hash-only
```

## Expected Behavior

1. **Shared** - base library, only its own files affect the hash
2. **Common** - hash depends on common files + shared hash
3. **Backend** - hash depends on backend files + common hash + shared hash + config files
4. **Frontend** - hash depends on frontend files + backend hash + shared hash + config directory
5. **Admin** - hash depends on admin files + frontend hash + backend hash + common hash + shared/README.md (but excludes it)

## Clean Up

To reset demo state:
```bash
# Remove generated files
rm -f demo/*/yeth.version
rm -rf demo/frontend/node_modules
rm -rf demo/backend/target
```

