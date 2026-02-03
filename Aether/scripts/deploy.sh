#!/bin/bash
set -e

# Configuration
BRANCH="master"
MAX_RETRIES=12
SLEEP_SECONDS=5
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Starting Aether Deployment Safe-Mode...${NC}"
cd "$PROJECT_DIR"

# 0. Safety Check
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "$BRANCH" ]; then
    echo -e "${RED}Error: Not on branch $BRANCH (Current: $CURRENT_BRANCH). Aborting.${NC}"
    exit 1
fi

# 1. Snapshot State (for Rollback)
OLD_COMMIT=$(git rev-parse HEAD)
echo -e "Current Commit: ${YELLOW}$OLD_COMMIT${NC}"

# Function: Rollback
rollback() {
    echo -e "${RED}Deployment Failed. Rolling back to $OLD_COMMIT...${NC}"
    git reset --hard "$OLD_COMMIT"
    
    echo "Restarting previous stable version..."
    docker-compose -f docker-compose.prod.yml up -d --build --remove-orphans
    
    echo -e "${RED}Rollback Complete. System reverted.${NC}"
    exit 1
}

# Trap errors
trap 'rollback' ERR

# 2. Update Code
echo "Pulling latest changes..."
git pull origin "$BRANCH"

# 3. Build & Deploy
echo "Building containers..."
# Determine config file
COMPOSE_FILE="docker-compose.prod.yml"

# Build first to fail early without taking down running containers
docker-compose -f "$COMPOSE_FILE" build

echo "Deploying..."
docker-compose -f "$COMPOSE_FILE" up -d --remove-orphans

# 4. Health Check
echo "Verifying deployment health..."
# Disable trap for manual health check handling
trap - ERR

count=0
healthy=false

while [ $count -lt $MAX_RETRIES ]; do
    # Check if all services are healthy
    # We filter for services that have a healthcheck. db, backend, frontend.
    # Note: docker-compose ps --format json is new, older versions differ.
    # We use a simple grep approach for compatibility.
    
    UNHEALTHY_COUNT=$(docker-compose -f "$COMPOSE_FILE" ps | grep -i "unhealthy" | wc -l)
    STARTING_COUNT=$(docker-compose -f "$COMPOSE_FILE" ps | grep -i "starting" | wc -l)
    
    if [ "$UNHEALTHY_COUNT" -gt 0 ]; then
        echo -e "${RED}Detected unhealthy containers.${NC}"
        rollback
    fi

    if [ "$STARTING_COUNT" -eq 0 ]; then
        # Ensure they are actually running (not exited)
        EXITED_COUNT=$(docker-compose -f "$COMPOSE_FILE" ps | grep -i "Exit" | wc -l)
        if [ "$EXITED_COUNT" -gt 0 ]; then
             echo -e "${RED}Containers exited unexpectedly.${NC}"
             rollback
        fi
        
        echo -e "${GREEN}All services operational.${NC}"
        healthy=true
        break
    fi

    echo -n "."
    sleep "$SLEEP_SECONDS"
    count=$((count + 1))
done

if [ "$healthy" = false ]; then
    echo -e "\n${RED}Timeout waiting for health checks.${NC}"
    rollback
fi

# 5. Cleanup
echo -e "\n${GREEN}Deployment Successful!${NC}"
echo "Cleaning up old images..."
docker image prune -f > /dev/null 2>&1

exit 0
