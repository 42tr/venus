#!/bin/bash

# Build the frontend
echo "Building frontend..."
cd frontend
npm install
npm run build
cd ..

# Build the Rust backend
echo "Building Rust backend..."
cargo build --release

echo "Build complete!"
echo "Binary location: target/release/venus"