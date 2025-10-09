#!/usr/bin/env bash

# Test script for yeth demo
# This script demonstrates various yeth features

set -e

YETH="../target/release/yeth"
ROOT="."

echo "=== Building yeth ==="
cd ..
cargo build --release
cd demo

echo ""
echo "=== Test 1: Show all application hashes ==="
$YETH --root $ROOT

echo ""
echo "=== Test 2: Show dependency graph ==="
$YETH --root $ROOT --show-graph

echo ""
echo "=== Test 3: Get specific app hash ==="
echo -n "Frontend hash: "
$YETH --root $ROOT --app frontend --hash-only

echo ""
echo "=== Test 4: Test dependency change propagation ==="
echo -n "Initial backend hash: "
INITIAL_HASH=$($YETH --root $ROOT --app backend --hash-only)
echo $INITIAL_HASH

echo "Modifying shared/utils.js..."
echo "// Test comment" >> shared/utils.js

echo -n "New backend hash: "
NEW_HASH=$($YETH --root $ROOT --app backend --hash-only)
echo $NEW_HASH

if [ "$INITIAL_HASH" != "$NEW_HASH" ]; then
    echo "✓ Hash changed correctly (dependency change detected)"
else
    echo "✗ Hash didn't change (unexpected)"
fi

# Revert change
git checkout shared/utils.js 2>/dev/null || sed -i '$d' shared/utils.js

echo ""
echo "=== Test 5: Test path dependency change ==="
echo -n "Initial backend hash: "
INITIAL_HASH=$($YETH --root $ROOT --app backend --hash-only)
echo $INITIAL_HASH

echo "Modifying config/database.json..."
cp config/database.json config/database.json.bak
echo '  "test": true' >> config/database.json

echo -n "New backend hash: "
NEW_HASH=$($YETH --root $ROOT --app backend --hash-only)
echo $NEW_HASH

if [ "$INITIAL_HASH" != "$NEW_HASH" ]; then
    echo "✓ Hash changed correctly (path dependency change detected)"
else
    echo "✗ Hash didn't change (unexpected)"
fi

# Revert
mv config/database.json.bak config/database.json

echo ""
echo "=== Test 6: Test exclusion patterns ==="
echo -n "Initial frontend hash: "
INITIAL_HASH=$($YETH --root $ROOT --app frontend --hash-only)
echo $INITIAL_HASH

echo "Creating node_modules (should be excluded)..."
mkdir -p frontend/node_modules/test-package
echo "console.log('test');" > frontend/node_modules/test-package/index.js

echo -n "Hash after adding node_modules: "
NEW_HASH=$($YETH --root $ROOT --app frontend --hash-only)
echo $NEW_HASH

if [ "$INITIAL_HASH" = "$NEW_HASH" ]; then
    echo "✓ Hash unchanged (node_modules correctly excluded)"
else
    echo "✗ Hash changed (exclusion not working)"
fi

# Cleanup
rm -rf frontend/node_modules

echo ""
echo "=== Test 7: Verify version file contents ==="
echo "Creating version files..."
$YETH --root $ROOT --write-versions > /dev/null

for app in shared common backend frontend admin; do
    if [ -f "$app/yeth.version" ]; then
        hash=$(cat "$app/yeth.version")
        computed=$($YETH --root $ROOT --app $app --hash-only)
        if [ "$hash" = "$computed" ]; then
            echo "✓ $app: version file matches computed hash"
        else
            echo "✗ $app: version file mismatch"
        fi
    else
        echo "✗ $app: version file not created"
    fi
done

# Cleanup version files
find . -name "yeth.version" -delete

echo ""
echo "=== Test 8: Save version files ==="
$YETH --root $ROOT --write-versions
echo "Version files created:"
find . -name "yeth.version" -type f | while read f; do
    echo "  $f: $(head -c 16 $f)..."
done

# Cleanup version files
find . -name "yeth.version" -delete

echo ""
echo "=== All tests completed ==="

