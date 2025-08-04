
#!/bin/bash

# Exit on error
set -e

# 1. Build the frontend
echo "Building frontend..."
cd frontend
npm install
npm run build
cd ..

# Check if frontend dist directory exists
if [ ! -d "frontend/dist" ]; then
  echo "Error: frontend/dist directory not found. Frontend build might have failed."
  exit 1
fi

# 2. Build the backend for linux/arm64
echo "Building backend for linux/arm64..."
mkdir -p build
GOOS=linux GOARCH=arm64 go build -o build/venus-linux-arm64 ./backend

echo "Build complete!"
