#!/bin/bash
set -e

# Ensure we are in the script's directory
cd "$(dirname "$0")"

echo "Starting Aether Backend..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo could not be found. Please ensure Rust is installed."
    exit 1
fi

# Set environment variables
export DATABASE_URL="sqlite://aether.db?mode=rwc"

# Start the backend server
echo "Running cargo run..."
exec cargo run --bin aether_backend -- "$@"
