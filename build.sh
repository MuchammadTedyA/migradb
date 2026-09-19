#!/bin/bash
set -e

echo "=== Building MigrDB ==="

# Build core Rust library
echo "Building Rust core library..."
cargo build --release

# Build Go bindings
echo "Building Go bindings..."
cd go
go build ./...
cd ..

# Build Node.js bindings
echo "Building Node.js bindings..."
cd npm
npm install
npm run build
cd ..

# Build Python bindings
echo "Building Python bindings..."
cd python
pip install maturin
maturin develop
cd ..

echo "=== Build Complete ==="
echo ""
echo "Run examples:"
echo "  Go:     go run go/example/main.go"
echo "  Node:   node npm/example.js"
echo "  Python: python python/example.py"
