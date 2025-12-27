@echo off
git status > status.txt 2>&1
if %errorlevel% neq 0 (
  echo Git failed >> status.txt
)
echo. >> status.txt
dir >> status.txt
echo SENTINEL >> status.txt
