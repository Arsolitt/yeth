# Installing yeth

## Binary Releases

Download the latest version for your platform from the [Releases](https://github.com/arsolitt/yeth/releases) page:

### Linux (x86_64)

```bash
# Standard build
wget https://github.com/arsolitt/yeth/releases/latest/download/yeth-x86_64-unknown-linux-gnu.tar.gz
tar xzf yeth-x86_64-unknown-linux-gnu.tar.gz
sudo mv yeth /usr/local/bin/

# Or static build (musl)
wget https://github.com/arsolitt/yeth/releases/latest/download/yeth-x86_64-unknown-linux-musl.tar.gz
tar xzf yeth-x86_64-unknown-linux-musl.tar.gz
sudo mv yeth /usr/local/bin/
```

### Linux (ARM64)

```bash
wget https://github.com/arsolitt/yeth/releases/latest/download/yeth-aarch64-unknown-linux-gnu.tar.gz
tar xzf yeth-aarch64-unknown-linux-gnu.tar.gz
sudo mv yeth /usr/local/bin/
```

### macOS (Intel)

```bash
curl -LO https://github.com/arsolitt/yeth/releases/latest/download/yeth-x86_64-apple-darwin.tar.gz
tar xzf yeth-x86_64-apple-darwin.tar.gz
sudo mv yeth /usr/local/bin/
```

### macOS (Apple Silicon)

```bash
curl -LO https://github.com/arsolitt/yeth/releases/latest/download/yeth-aarch64-apple-darwin.tar.gz
tar xzf yeth-aarch64-apple-darwin.tar.gz
sudo mv yeth /usr/local/bin/
```

### Windows

Download `yeth-x86_64-pc-windows-msvc.zip` from the releases page and extract it to your desired directory.

## Verifying Checksums

Each release includes `.sha256` files for integrity verification:

```bash
# Linux/macOS
sha256sum -c yeth-x86_64-unknown-linux-gnu.tar.gz.sha256

# Windows (PowerShell)
$hash = (Get-FileHash yeth-x86_64-pc-windows-msvc.zip -Algorithm SHA256).Hash.ToLower()
$expected = (Get-Content yeth-x86_64-pc-windows-msvc.zip.sha256).Split()[0]
if ($hash -eq $expected) { "OK" } else { "FAILED" }
```

## Docker

```bash
# From GitHub Container Registry
docker pull ghcr.io/arsolitt/yeth:latest

# Usage
docker run --rm -v $(pwd):/workspace ghcr.io/arsolitt/yeth:latest [arguments]
```

## Building from Source

```bash
git clone https://github.com/arsolitt/yeth.git
cd yeth
cargo build --release
sudo mv target/release/yeth /usr/local/bin/
```

## Verifying Installation

```bash
yeth --version
```
