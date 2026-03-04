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

- [x] **SAFE-01**: macOS-specific destructive commands (csrutil, dscl, nvram, security, tmutil, spctl, launchctl, diskutil, srm, pfctl) are detected
- [x] **SAFE-02**: Linux-specific destructive commands (systemctl, iptables, nft, userdel, groupdel, parted, gdisk, wipefs, lvm, cryptsetup, crontab, modprobe, swapoff, truncate) are detected
- [x] **SAFE-03**: Container/orchestration destructive commands (docker prune/rm/volume/network, kubectl delete, helm uninstall) are detected
- [x] **SAFE-04**: Package manager uninstall commands (apt, brew, pip, npm, cargo, choco, pacman, dnf, snap) are detected
- [x] **SAFE-05**: Config file overwrites via redirect (> ~/.bashrc, > /etc/hosts, > ~/.ssh/config, > /etc/passwd, etc.) are detected
- [x] **SAFE-06**: All patterns are organized with clear comment section headers and existing patterns are preserved/consolidated

## v0.3.0 Requirements

Requirements for this milestone. Each maps to roadmap phases.

### CI/CD Release Pipeline

- [ ] **CICD-01**: Single `release.yml` workflow triggered by `v*` tag push builds macOS and Windows artifacts
- [ ] **CICD-02**: macOS build produces signed, notarized, stapled universal DMG using parameterized `build-dmg.sh`
- [ ] **CICD-03**: Windows build produces unsigned NSIS installer with conditional signing block for future enablement
- [ ] **CICD-04**: GitHub Release auto-published with both platform artifacts and SHA256 checksums
- [ ] **CICD-05**: Apple signing credentials (p12 certificate, notarization secrets) migrated from local keychain to GitHub Secrets with step-by-step documentation
- [ ] **CICD-06**: `build-dmg.sh` parameterized via environment variables -- version derived from tag, keychain profile configurable for CI

## Future Requirements

None currently deferred.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Persistent position across relaunches | User explicitly wants session-scoped only |
| Per-monitor position memory | One global position chosen |
| Windows overlay fixes | macOS-only milestone |
| Snap-to-edge positioning | Not requested, keep simple |
| PR build verification | Future improvement -- compile-only, no artifacts |
| Windows OV/EV code signing | Purchase when distribution warrants it |
| Auto-updater (tauri-plugin-updater) | Separate phase |
| Linux builds | Separate phase |

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
| CICD-01 | Phase 20 | Planned |
| CICD-02 | Phase 20 | Planned |
| CICD-03 | Phase 20 | Planned |
| CICD-04 | Phase 20 | Planned |
| CICD-05 | Phase 20 | Planned |
| CICD-06 | Phase 20 | Planned |

**Coverage:**
- v0.2.2 requirements: 5 total, all complete
- v0.2.3 requirements: 6 total
- v0.3.0 requirements: 6 total
- Mapped to phases: 17
- Unmapped: 0

---
*Requirements defined: 2026-03-03*
*Last updated: 2026-03-04 after Phase 20 planning*
