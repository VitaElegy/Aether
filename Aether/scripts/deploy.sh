#!/bin/bash
set -e

echo "Starting Aether Deployment..."

# 1. Pull latest code
echo "Pulling latest changes from git..."
git pull origin main

# 2. Rebuild and Restart containers
echo "Rebuilding and restarting containers..."
# Use prod config
docker-compose -f docker-compose.prod.yml up -d --build --remove-orphans

# 3. Cleanup unused images
echo "Cleaning up old images..."
docker image prune -f

echo "Deployment Complete! System running on port 80."
