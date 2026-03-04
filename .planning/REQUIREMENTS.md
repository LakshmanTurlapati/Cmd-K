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

**Coverage:**
- v0.2.2 requirements: 5 total
- Mapped to phases: 5
- Unmapped: 0

---
*Requirements defined: 2026-03-03*
*Last updated: 2026-03-03 after roadmap creation*
