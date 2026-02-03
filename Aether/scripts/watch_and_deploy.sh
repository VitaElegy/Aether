#!/bin/bash

# Configuration
BRANCH="main"
INTERVAL=60 # Check every 60 seconds
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Starting Aether Watchtower..."
echo "Watching branch '$BRANCH' in $PROJECT_DIR"

while true; do
    cd "$PROJECT_DIR"
    
    # Fetch remote to detect changes
    git fetch origin "$BRANCH" > /dev/null 2>&1
    
    # Compare local HEAD with remote HEAD
    LOCAL=$(git rev-parse HEAD)
    REMOTE=$(git rev-parse "origin/$BRANCH")
    
    if [ "$LOCAL" != "$REMOTE" ]; then
        echo "[$(date)] Changes detected! Updating..."
        echo "Local:  $LOCAL"
        echo "Remote: $REMOTE"
        
        # Trigger Deployment (which handles pull + build)
        bash scripts/deploy.sh
        
        echo "[$(date)] Update complete. Resuming watch."
    else
        # Optional: Heartbeat log
        # echo "[$(date)] No changes."
        true
    fi
    
    sleep $INTERVAL
done
