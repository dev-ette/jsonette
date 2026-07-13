#!/bin/bash
# Exit immediately if a command exits with a non-zero status
set -e

# Get the directory of this script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
WORKSPACE_DIR="$SCRIPT_DIR/.."

# Navigate to the workspace root directory
cd "$WORKSPACE_DIR"

echo "Building Rust engine documentation..."
cargo doc --package jsonette-core --no-deps

echo "Cleaning up old documentation in docs/engine-docs..."
rm -rf "$SCRIPT_DIR/engine-docs"
mkdir -p "$SCRIPT_DIR/engine-docs"

echo "Copying generated documentation..."
cp -R target/doc/ "$SCRIPT_DIR/engine-docs/"

echo "--------------------------------------------------------"
echo "Documentation successfully generated at docs/engine-docs/"
echo "Open docs/engine-docs/jsonette/index.html to view it."
echo "--------------------------------------------------------"
