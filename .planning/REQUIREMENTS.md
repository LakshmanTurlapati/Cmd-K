# Requirements: CMD+K

**Defined:** 2026-03-03
**Core Value:** The overlay must appear on top of the active application and feel instant

## v0.2.2 Requirements

Requirements for this milestone. Each maps to roadmap phases.

### Overlay Z-Order

- [x] **ZORD-01**: System permission and accessibility dialogs can appear above the CMD+K overlay
- [x] **ZORD-02**: System UI elements (Notification Center, Spotlight, other system overlays) can appear above the CMD+K overlay

### Overlay Positioning

- [x] **OPOS-01**: User can drag the overlay to reposition it on screen
- [x] **OPOS-02**: Overlay reopens at the last dragged position within the same app session
- [x] **OPOS-03**: Overlay position resets to default on app relaunch

## v0.2.3 Requirements

Requirements for this milestone. Each maps to roadmap phases.

### Destructive Command Coverage

- [ ] **SAFE-01**: macOS-specific destructive commands (csrutil, dscl, nvram, security, tmutil, spctl, launchctl, diskutil, srm, pfctl) are detected
- [ ] **SAFE-02**: Linux-specific destructive commands (systemctl, iptables, nft, userdel, groupdel, parted, gdisk, wipefs, lvm, cryptsetup, crontab, modprobe, swapoff, truncate) are detected
- [ ] **SAFE-03**: Container/orchestration destructive commands (docker prune/rm/volume/network, kubectl delete, helm uninstall) are detected
- [ ] **SAFE-04**: Package manager uninstall commands (apt, brew, pip, npm, cargo, choco, pacman, dnf, snap) are detected
- [ ] **SAFE-05**: Config file overwrites via redirect (> ~/.bashrc, > /etc/hosts, > ~/.ssh/config, > /etc/passwd, etc.) are detected
- [ ] **SAFE-06**: All patterns are organized with clear comment section headers and existing patterns are preserved/consolidated

## Future Requirements

None currently deferred.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Persistent position across relaunches | User explicitly wants session-scoped only |
| Per-monitor position memory | One global position chosen |
| Windows overlay fixes | macOS-only milestone |
| Snap-to-edge positioning | Not requested, keep simple |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| ZORD-01 | Phase 17 | Complete |
| ZORD-02 | Phase 17 | Complete |
| OPOS-01 | Phase 18 | Complete |
| OPOS-02 | Phase 18 | Complete |
| OPOS-03 | Phase 18 | Complete |
| SAFE-01 | Phase 19 | Planned |
| SAFE-02 | Phase 19 | Planned |
| SAFE-03 | Phase 19 | Planned |
| SAFE-04 | Phase 19 | Planned |
| SAFE-05 | Phase 19 | Planned |
| SAFE-06 | Phase 19 | Planned |

**Coverage:**
- v0.2.2 requirements: 5 total, all complete
- v0.2.3 requirements: 6 total
- Mapped to phases: 11
- Unmapped: 0

---
*Requirements defined: 2026-03-03*
*Last updated: 2026-03-04 after Phase 19 planning*
