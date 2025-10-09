# Quick Start Guide

## 🚀 Run Demo in 30 seconds

```bash
# From project root
cargo build --release
cd demo
./quick-test.sh
```

## 📊 What you'll see

The demo includes **5 applications** with various dependency types:

1. **shared** - Base library (no dependencies)
2. **common** - Utilities (→ shared)
3. **backend** - API service (→ common, shared, config files)
4. **frontend** - Web app (→ backend, shared, config dir)
5. **admin** - Admin panel (→ frontend, backend, common, shared dir)

## 🧪 Run Full Tests

```bash
./test.sh
```

Tests verify:
- ✅ Hash calculation for all apps
- ✅ Dependency graph building
- ✅ Change propagation through dependencies
- ✅ Path dependency detection
- ✅ Exclusion patterns (node_modules, dist, etc.)
- ✅ Version file generation

## 📝 Try Individual Commands

```bash
# Show all hashes
../target/release/yeth --root .

# Show dependency graph
../target/release/yeth --root . --show-graph

# Get specific app hash
../target/release/yeth --root . --app frontend --hash-only

# Save version files
../target/release/yeth --root . --write-versions
```

## 🔍 Explore More

- **[README.md](README.md)** - Full documentation
- **[EXAMPLES.md](EXAMPLES.md)** - Copy-paste command examples
- **[test.sh](test.sh)** - Automated test scenarios

## 💡 Quick Experiments

### 1. See hash propagation
```bash
# Get frontend hash
../target/release/yeth --root . --app frontend --hash-only

# Modify dependency
echo "// change" >> shared/utils.js

# Check hash again (will be different!)
../target/release/yeth --root . --app frontend --hash-only

# Revert
git checkout shared/utils.js
```

### 2. Test exclusions
```bash
# Create excluded directory
mkdir -p frontend/node_modules
echo "test" > frontend/node_modules/test.js

# Hash won't change (node_modules is excluded)
../target/release/yeth --root . --app frontend --hash-only

# Cleanup
rm -rf frontend/node_modules
```

### 3. CI/CD simulation
```bash
# Save current hash
HASH=$(../target/release/yeth --root . --app backend --hash-only)
echo $HASH > .last-build-hash

# Make change
echo "// update" >> backend/main.rs

# Compare hashes
NEW_HASH=$(../target/release/yeth --root . --app backend --hash-only)
if [ "$HASH" != "$NEW_HASH" ]; then
    echo "Changes detected - rebuild needed!"
fi

# Cleanup
git checkout backend/main.rs
rm .last-build-hash
```

## 📦 Demo Structure

```
demo/
├── shared/           # JS library - base dependency
├── common/           # Rust library - depends on shared
├── config/           # Config files - used as path dependencies
├── backend/          # Rust API - depends on common, shared, config
├── frontend/         # Web app - depends on backend, shared, config
└── admin/            # Admin panel - depends on all apps + shared dir
```

## 🎯 Key Features Demonstrated

- **Application dependencies** - Apps depending on other apps
- **File dependencies** - Direct file references (`config/database.json`)
- **Directory dependencies** - Entire directory tracking (`../config`)
- **Exclusion patterns** - Ignore build artifacts (`node_modules`, `dist`, `target`)
- **Circular dependency detection** - Automatic validation
- **Hash propagation** - Changes cascade through dependency chain
- **Version tracking** - Save hashes to `yeth.version` files

---

**Need help?** Check [README.md](README.md) for detailed documentation.

