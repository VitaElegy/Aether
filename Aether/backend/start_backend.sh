#!/bin/bash

echo "Starting Aether Backend..."

# Set environment variables
export DATABASE_URL="sqlite://aether.db?mode=rwc"

# Start the backend server
cargo run
