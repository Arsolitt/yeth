# yeth

A utility for building dependency graphs between applications and calculating their hashes.

## How it works

1. Scans directories for `yeth.toml` files
2. Builds a dependency graph between applications
3. Checks for circular dependencies
4. Calculates the hash of each application including its dependencies' hashes
5. Outputs the results

## Installation

```bash
cargo build --release
```

Binary will be in `target/release/yeth`

## Usage

### Basic usage

Output hashes of all applications:

```bash
yeth
```

### Output hash of specific application

```bash
yeth --app my-app
```

### Output only hash (without name)

Useful for scripts:

```bash
yeth --app my-app --hash-only
```

### Specify root directory

```bash
yeth --root /path/to/monorepo
```

### Show dependency graph

```bash
yeth --show-graph
```

### Show statistics

```bash
yeth --verbose
```

### Save version files

Save each application's hash to `yeth.version` file next to `yeth.toml`:

```bash
yeth --write-versions
```

## Configuration format

Create a `yeth.toml` file in the root of each application:

```toml
[app]
dependencies = ["app1", "app2"]
```

If the application has no dependencies:

```toml
[app]
dependencies = []
```

### Dependency types

You can specify two types of dependencies:

1. **Dependencies on other applications** (names without slashes):
```toml
[app]
dependencies = ["app1", "backend", "shared"]
```

2. **Dependencies on files and directories** (relative paths):
```toml
[app]
dependencies = [
    "../shared/config.json",      # file
    "./vendor",                    # directory
    "../../../root-config.yaml"   # path up the tree
]
```

3. **Mixed dependencies**:
```toml
[app]
dependencies = [
    "app1",                    # application dependency
    "../shared/utils",         # directory dependency
    "./config.json"            # file dependency
]
```

**Type determination rule:**
- If string contains `/` or starts with `.` → it's a path to file/directory
- Otherwise → it's an application name

**Important:** Paths are resolved relative to the application directory (where `yeth.toml` is located).

### Excluding files from hashing

You can specify files and directories to exclude from application hash calculation:

```toml
[app]
dependencies = ["app1"]
exclude = [
    "node_modules",           # ignore node_modules directory
    "dist",                   # ignore dist directory
    "target",                 # ignore target directory
    ".env",                   # ignore .env file
    "tests",                  # ignore everything in tests/
    "src/generated"           # ignore src/generated/
]
```

**How exclusion works:**
- Patterns are checked relative to application root
- You can specify directory name (`node_modules`) — will be excluded wherever it appears
- You can specify path (`src/generated`) — will exclude specific path
- Prefix matching: if path starts with pattern, it's excluded
- **Important:** Patterns with paths (`../shared/README.md`) apply globally — will exclude files even inside dependencies

**Examples:**

```toml
# Basic configuration without exclusions
[app]
dependencies = []

# With local file exclusions
[app]
dependencies = ["backend"]
exclude = ["node_modules", "dist", "tmp"]

# Excluding files from dependencies
[app]
dependencies = ["../shared"]
exclude = ["../shared/README.md", "../shared/docs"]

# Combined exclusion
[app]
dependencies = ["../shared", "../common"]
exclude = [
    "node_modules",              # local exclusion
    "../shared/README.md",       # file exclusion from dependency
    "../common/tests"            # directory exclusion from dependency
]
```

## Examples

### Project structure

#### Example 1: Only application dependencies

```
monorepo/
├── app1/
│   ├── yeth.toml      # dependencies = []
│   └── src/
├── app2/
│   ├── yeth.toml      # dependencies = ["app1"]
│   └── src/
└── app3/
    ├── yeth.toml      # dependencies = ["app1", "app2"]
    └── src/
```

#### Example 2: With file and directory dependencies

```
monorepo/
├── shared/
│   ├── config.json
│   └── utils/
├── apps/
│   ├── frontend/
│   │   ├── yeth.toml      # dependencies = ["backend", "../../shared/config.json"]
│   │   │                  # exclude = ["node_modules", "dist"]
│   │   ├── node_modules/
│   │   ├── dist/
│   │   └── src/
│   └── backend/
│       ├── yeth.toml      # dependencies = ["../../shared/utils"]
│       │                  # exclude = ["target"]
│       ├── target/
│       └── src/
└── config.yaml
```

**frontend/yeth.toml config:**
```toml
[app]
dependencies = ["backend", "../../shared/config.json"]
exclude = ["node_modules", "dist", ".next"]
```

**backend/yeth.toml config:**
```toml
[app]
dependencies = ["../../shared/utils"]
exclude = ["target", "*.log"]
```

### Output examples

#### Normal output

```bash
$ yeth
a1b2c3d4... app1
e5f6g7h8... app2
i9j0k1l2... app3

Execution time: 123.45ms
Applications processed: 3
```

#### Dependency graph

```bash
$ yeth --show-graph
Dependency graph:

app1
  └─ (no dependencies)

app2
  ├─ app1 (app)
  └─ ../shared/config.json (file)

app3
  ├─ app1 (app)
  ├─ app2 (app)
  └─ ../shared/utils (dir)
```

### Usage in CI/CD

Get application hash to determine if rebuild is needed:

```bash
#!/bin/bash
APP_HASH=$(yeth --app my-app --hash-only --verbose)
echo "Current hash: $APP_HASH"

# Compare with saved hash and decide if rebuild is needed
if [ "$APP_HASH" != "$LAST_BUILD_HASH" ]; then
    echo "Changes detected, starting build..."
    # build commands here
fi
```

### Practical example 1: Simple exclusion

Project structure:
```
example/
├── app1/
│   ├── yeth.toml          # dependencies = [], exclude = ["node_modules", "dist"]
│   ├── main.js
│   ├── node_modules/      # excluded from hash
│   └── dist/              # excluded from hash
└── app2/
    ├── yeth.toml          # dependencies = ["app1"], exclude = ["target"]
    ├── main.rs
    └── target/            # excluded from hash
```

**Important:** Files in `node_modules`, `dist` and `target` don't affect hash, as they're specified in `exclude`.

### Practical example 2: Excluding files from dependencies

Structure:
```
example/
├── catalog/
│   ├── yeth.toml          # dependencies = ["../shared"]
│   │                      # exclude = ["../shared/README.md"]
│   └── index.js
└── shared/
    ├── yeth.toml
    ├── utils.js
    └── README.md          # excluded from catalog hash, but NOT excluded from shared hash
```

**catalog/yeth.toml config:**
```toml
[app]
dependencies = ["../shared"]
exclude = ["../shared/README.md", "node_modules"]
```

**Result:**
- Changing `shared/README.md` → `catalog` hash **does NOT change** ✅
- Changing `shared/README.md` → `shared` hash **changes** (it doesn't have this exclusion)
- Changing `shared/utils.js` → `catalog` hash **changes** ✅

Execution:
```bash
$ yeth --root example
47aa9e986c6e4c0b7bd839d97eda81700fccc8575e1cfa8cf7ce70809c4bfb1e catalog
d98a899314cd6581de6446f1a427a9822013b3065a92a38f61d381571c86da7d shared

# Modify shared/README.md
$ echo "update" >> shared/README.md
$ yeth --root example
47aa9e986c6e4c0b7bd839d97eda81700fccc8575e1cfa8cf7ce70809c4bfb1e catalog  ← unchanged
00214c62f5d76e98dac137675059581576eeabfc8d084dc8b6206f84dd84f692 shared  ← changed

# Modify shared/utils.js
$ echo "update" >> shared/utils.js  
$ yeth --root example
54010998be564b7a736a48e418084ec3247c23e8d2d5d1ba8c4065d75ea988fa catalog  ← changed
25116e4ece02de6be08a5093f3b867092e0b1df4713f087470bb932afe5785bb shared  ← changed
```

## Command line options

```
Options:
  -r, --root <ROOT>        Root directory to search for applications [default: .]
  -a, --app <APP>          Name of specific application to output hash for
  -H, --hash-only          Show only hash without application name
  -v, --verbose            Show execution time statistics
  -g, --show-graph         Show dependency graph
  -w, --write-versions     Save each application's hash to yeth.version next to yeth.toml
  -h, --help               Print help
```

## Architecture

The project is split into modules:

- `cli.rs` - Command line argument parsing (clap)
- `config.rs` - Reading and parsing configuration files
- `graph.rs` - Building dependency graph and topological sorting
- `hash.rs` - Calculating directory and application hashes
- `main.rs` - Entry point and work coordination

## Hash calculation algorithm

1. For each application, calculate its own hash (SHA256 of all files in directory)
2. For path dependencies, calculate file or directory hash
3. Applications are processed in topological order (by application dependencies)
4. Final hash = SHA256(own_hash + dependency_hash_1 + ... + dependency_hash_N)

**Important points:**
- Changes in any dependency (application, file, directory) will affect the hash of all applications depending on it
- File/directory dependencies don't participate in topological sorting (they can't be circular)
- Path dependencies are checked for existence at program start
- System files (`.git`, `.DS_Store`, `yeth.version`) are automatically ignored
- Additional files can be excluded via the `exclude` field in config
