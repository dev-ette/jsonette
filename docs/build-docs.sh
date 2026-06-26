#!/bin/bash
# Exit immediately if a command exits with a non-zero status
set -e

# Get the directory of this script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Navigate to the engine directory
cd "$SCRIPT_DIR/../engine"

echo "Building Rust engine documentation..."
cargo doc --no-deps

echo "Cleaning up old documentation in docs/engine-docs..."
rm -rf "$SCRIPT_DIR/engine-docs"
mkdir -p "$SCRIPT_DIR/engine-docs"

echo "Copying generated documentation..."
cp -R target/doc/ "$SCRIPT_DIR/engine-docs/"

echo "--------------------------------------------------------"
echo "Documentation successfully generated at docs/engine-docs/"
echo "Open docs/engine-docs/jsonette/index.html to view it."
echo "--------------------------------------------------------"
