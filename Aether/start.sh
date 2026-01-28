#!/bin/bash
# Root Shim for Aether Backend
# Forwards to the actual script in scripts/setup/start.sh (or backend/start.sh if we kept it there)
# Based on the plan, we moved scripts. 
# Let's assume we have a start_dev.sh or similar in scripts/backend or scripts/setup.
# Since I haven't seen a start.sh in the file list, I'll assume the user wants one created or I need to find the old one.
# The user said "./start.sh之类的工具".
# I will point this to `scripts/backend/run_dev.sh` if it exists, or just `cargo run`.
# I'll stick to a generic message or try to locate the real start script in the next step if I can't find it.
# For now, I'll make it a placeholder that points to the new location.

echo "Redirecting to scripts/backend/..."
# checks if scripts/backend/start.sh exists?
# I'll update this after verifying the actual start script name.
echo "Error: Please verify the actual start script location in scripts/backend/"
exit 1
