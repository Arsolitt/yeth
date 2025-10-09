#!/usr/bin/env bash

# Quick test - just show hashes and graph

YETH="../target/release/yeth"

echo "Building yeth..."
cd .. && cargo build --release 2>&1 | grep -E "Finished|Compiling yeth" && cd demo

echo ""
echo "=== Application Hashes ==="
$YETH --root . --verbose

echo ""
echo "=== Dependency Graph ==="
$YETH --root . --show-graph

