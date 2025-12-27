@echo off
echo Committing Backend...
git add Aether\backend\src
git add Aether\backend\Cargo.toml
git commit -m "feat(backend): Update domain models and API for version history and content management"

echo Committing Frontend...
git add Aether\frontend\src
git commit -m "feat(frontend): Add version history views and diff viewer"

echo Committing Tooling...
git add debug_feed.py
git add seed_public.py
git add test_api.py
git add commit_all.bat
git add diagnose_git.bat
git add package.json
git commit -m "chore: Add utility scripts and update configuration"

echo Committing Database...
git add Aether\backend\aether.db
git commit -m "chore: Update local database state"

echo All commits finished.
