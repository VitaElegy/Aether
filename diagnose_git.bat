@echo off
git status > git_status.txt 2>&1
git log -1 >> git_status.txt 2>&1
echo DONE >> git_status.txt
