#!/bin/bash

# Configuration
BRANCH="master"
INTERVAL=60
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_FILE="$PROJECT_DIR/deploy.log"
PID_FILE="$PROJECT_DIR/watch_deploy.pid"
MAX_LOG_SIZE=$((10 * 1024 * 1024)) # 10MB

# Process Locking
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p "$PID" > /dev/null; then
        echo "Watcher is already running (PID: $PID). Exiting."
        exit 1
    fi
fi
echo $$ > "$PID_FILE"

# Cleanup lock on exit
trap 'rm -f "$PID_FILE"' EXIT

# Redirect stdout/stderr to log file
exec >> "$LOG_FILE" 2>&1

echo "=========================================="
echo "Starting Aether Watchtower at $(date)"
echo "Watching branch '$BRANCH' in $PROJECT_DIR"
echo "=========================================="

# Log Rotation Function
rotate_logs() {
    if [ -f "$LOG_FILE" ]; then
        SIZE=$(stat -f%z "$LOG_FILE" 2>/dev/null || stat -c%s "$LOG_FILE")
        if [ "$SIZE" -gt "$MAX_LOG_SIZE" ]; then
            mv "$LOG_FILE" "$LOG_FILE.1"
            echo "[$(date)] Logs rotated." > "$LOG_FILE"
        fi
    fi
}

while true; do
    cd "$PROJECT_DIR" || exit 1
    
    # 1. Rotate logs if needed
    rotate_logs

    # 2. Check Git
    # Fetch remote silently
    if ! git fetch origin "$BRANCH" > /dev/null 2>&1; then
        echo "[$(date)] Warn: Git fetch failed. Retrying in next interval."
        sleep "$INTERVAL"
        continue
    fi

    LOCAL=$(git rev-parse HEAD)
    REMOTE=$(git rev-parse "origin/$BRANCH")

    if [ "$LOCAL" != "$REMOTE" ]; then
        echo "[$(date)] Update detected!"
        echo "Local:  $LOCAL"
        echo "Remote: $REMOTE"
        
        # 3. Trigger Deployment
        echo "[$(date)] Starting Deployment Script..."
        if bash scripts/deploy.sh; then
            echo "[$(date)] Deployment finished successfully."
        else
            echo "[$(date)] Deployment FAILED. Please check logs."
        fi
    else
        # Heartbeat every hour (optional, to keep logs quiet)
        # echo "[$(date)] No changes."
        true
    fi

    sleep "$INTERVAL"
done
