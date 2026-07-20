#!/bin/bash
set -e

# Export paths for Cargo and Homebrew
export PATH="$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"

echo "⚙️  Building Rust Engine & generating Swift FFI bindings..."
cd ..
cargo build --release --manifest-path engine/Cargo.toml

mkdir -p macos/Generated
cargo run --release --bin uniffi-bindgen --manifest-path engine/Cargo.toml -- generate \
  --library target/release/libjsonette_core.dylib \
  --language swift \
  --out-dir macos/Generated

if [ -f "macos/Generated/jsonette_coreFFI.modulemap" ]; then
  mv "macos/Generated/jsonette_coreFFI.modulemap" "macos/Generated/module.modulemap"
fi

echo "⚙️  Generating Xcode Project..."
cd macos
xcodegen

echo "✅ Setup complete! You can now open macos/jsonette.xcodeproj in Xcode and hit Build."
