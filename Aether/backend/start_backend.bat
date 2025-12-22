@echo off
echo Starting Aether Backend...

REM Set environment variables
set DATABASE_URL=sqlite://aether.db?mode=rwc

REM Start the backend server
cargo run
