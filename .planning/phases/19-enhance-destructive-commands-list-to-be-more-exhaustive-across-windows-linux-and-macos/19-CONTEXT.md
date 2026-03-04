# Phase 19: Enhance Destructive Commands List - Context

**Gathered:** 2026-03-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Expand the regex pattern list in `safety.rs` to comprehensively cover destructive commands across macOS, Linux, and Windows. Add new categories (containers, package managers, config file overwrites). No changes to the badge UX, severity model, or safety architecture — just more patterns in the existing `DESTRUCTIVE_PATTERNS` RegexSet.

</domain>

<decisions>
## Implementation Decisions

### Command categories to add
- **macOS-specific**: csrutil disable (SIP), dscl delete (users/groups), nvram delete, security delete-keychain, tmutil disable (Time Machine), spctl --master-disable (Gatekeeper), launchctl remove, diskutil eraseDisk/partitionDisk/eraseVolume, srm, pfctl flush
- **Linux-specific**: systemctl disable/mask, iptables -F, nft flush ruleset, userdel, groupdel, parted rm, gdisk, wipefs, lvm ops (lvremove, vgremove, pvremove), cryptsetup luksErase, crontab -r, modprobe -r, swapoff -a, truncate -s 0
- **Container/orchestration**: docker system prune -a, docker rm -f, docker volume rm, docker network rm, kubectl delete (all resource types), helm uninstall
- **Package managers (all major)**: apt purge, apt autoremove, brew uninstall, pip uninstall, npm uninstall -g, cargo uninstall, choco uninstall, pacman -Rns, dnf remove, snap remove
- **Config file overwrites**: Redirect to config files (> ~/.bashrc, > /etc/hosts, > ~/.ssh/config, > /etc/passwd, etc.)

### Severity tiering
- **Single tier only** — every destructive command gets the same red "Destructive" badge
- No severity levels, no color differentiation, no different badge text
- AI explanation is uniform — just explains what the command does, no severity context

### False positive tolerance
- **Err on the side of caution** — flag anything potentially destructive even if sometimes routine
- User can dismiss the badge with one click, so over-warning is acceptable; under-warning is not
- Flag all `kubectl delete` regardless of resource type (pod, namespace, etc.) — AI explanation will differentiate
- `sudo` is NOT flagged by itself — only when combined with a destructive command (matches current behavior)
- Config file redirects ARE flagged (> ~/.bashrc, > /etc/hosts, etc.)

### Pattern organization
- **Commented sections in one flat array** — no architecture changes, no separate files
- Clear comment headers per section: `// === macOS ===`, `// === Linux ===`, `// === Windows ===`, `// === Containers ===`, `// === Package managers ===`, `// === Config overwrites ===`
- **Group comments only** — comment at section level, not per-pattern
- **All patterns active on all platforms** — no OS-specific filtering. A macOS user may generate Linux commands for remote servers.
- **No test suite** — manual verification during development is sufficient

### Claude's Discretion
- Exact regex patterns for each new command (word boundaries, flag combinations, case sensitivity)
- Whether to refine or tighten any existing patterns while adding new ones
- Ordering of patterns within each section
- Whether any existing patterns have become redundant and can be consolidated

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. The user's core ask: make the destructive command list exhaustive across all three platforms, plus containers and package managers, while keeping the single-badge UX unchanged.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `safety.rs`: Contains `DESTRUCTIVE_PATTERNS` as `Lazy<RegexSet>` with ~80 patterns, `check_destructive()` Tauri command, and `get_destructive_explanation()` AI-powered explanation
- `DestructiveBadge.tsx`: Frontend component — no changes needed, already supports single-tier display
- `Overlay.tsx`: Triggers `check_destructive()` on streaming text — no changes needed

### Established Patterns
- `once_cell::sync::Lazy<RegexSet>` — compiled once, zero allocation on subsequent checks
- Word boundaries (`\b`) to avoid false positives on substrings
- Case-insensitive `(?i)` for SQL and Windows commands
- Comment headers grouping patterns by category (// File destruction, // Git force operations, etc.)

### Integration Points
- Only `safety.rs` needs modification — specifically the `DESTRUCTIVE_PATTERNS` array
- No frontend changes required
- No IPC changes required
- No new dependencies required

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 19-enhance-destructive-commands-list-to-be-more-exhaustive-across-windows-linux-and-macos*
*Context gathered: 2026-03-04*
