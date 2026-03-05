---
status: complete
phase: 19-enhance-destructive-commands-list-to-be-more-exhaustive-across-windows-linux-and-macos
source: 19-01-SUMMARY.md
started: 2026-03-04T19:00:00Z
updated: 2026-03-04T19:02:00Z
---

## Current Test

[testing complete]

## Tests

### 1. macOS destructive commands trigger badge
expected: Open CMD+K and have it generate or display a macOS system command like `csrutil disable`, `diskutil eraseDisk JHFS+ Untitled /dev/disk2`, or `tmutil disable`. The red "Destructive" badge should appear on the command.
result: pass

### 2. Linux destructive commands trigger badge
expected: Have CMD+K display a Linux command like `systemctl disable nginx`, `iptables -F`, or `userdel olduser`. The red "Destructive" badge should appear.
result: pass

### 3. Container/orchestration commands trigger badge
expected: Have CMD+K display a container command like `docker system prune -a`, `kubectl delete namespace production`, or `helm uninstall my-release`. The red "Destructive" badge should appear.
result: pass

### 4. Package manager uninstall commands trigger badge
expected: Have CMD+K display a package manager removal like `brew uninstall node`, `pip uninstall requests`, or `npm uninstall -g typescript`. The red "Destructive" badge should appear.
result: pass

### 5. Config file overwrite commands trigger badge
expected: Have CMD+K display a config redirect like `> ~/.bashrc` or `> /etc/hosts`. The red "Destructive" badge should appear.
result: pass

### 6. Existing patterns still work (no regressions)
expected: Have CMD+K display classic destructive commands like `rm -rf /`, `git push --force`, or `DROP TABLE users`. The red "Destructive" badge should still appear as before.
result: pass

### 7. Non-destructive commands do NOT trigger badge
expected: Have CMD+K display safe commands like `ls -la`, `git status`, `docker ps`, `kubectl get pods`, or `npm install express`. No "Destructive" badge should appear.
result: pass

## Summary

total: 7
passed: 7
issues: 0
pending: 0
skipped: 0

## Gaps

[none]
