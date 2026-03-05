# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v0.2.4 -- Overlay UX, Safety & CI/CD

**Shipped:** 2026-03-04
**Phases:** 4 | **Plans:** 5

### What Was Built
- Overlay z-order fix (Floating level) and drag-to-reposition with session-scoped memory
- Cross-platform destructive command safety expanded from ~80 to 150 patterns
- Windows Terminal ConPTY shell detection fix
- Automated CI/CD pipeline with GitHub Actions for signed macOS DMG and Windows NSIS builds
- Parameterized build script and secrets setup documentation

### What Worked
- Small focused phases (17-18 overlay UX, 19 safety, 20 CI/CD) kept each phase under 15 minutes of execution
- Phase 17 was a single-line change with high impact -- good scoping
- Build script parameterization (Plan 20-01) before workflow creation (Plan 20-02) was the right sequence
- Yolo mode with plan-check enabled: fast execution with quality gates where they matter

### What Was Inefficient
- Requirements were split across three sub-milestone labels (v0.2.2, v0.2.3, v0.3.0) but shipped as one release (v0.2.4) -- cleaner to define a single milestone scope upfront
- Phases 19-20 were added incrementally after v0.2.2 was already defined -- milestone scope creep required re-labeling at archive time
- No milestone audit performed before archiving -- skipped to move faster

### Patterns Established
- PanelLevel::Floating as the standard overlay level for macOS utility panels
- // === Section === headers for organizing large pattern arrays
- Env var with fallback pattern for CI/local dual-path scripts
- Tag-triggered release pipeline with artifact pass-through architecture

### Key Lessons
1. Define milestone scope upfront rather than accumulating phases post-hoc -- avoids version label confusion at archive time
2. Single-line changes can have outsized impact (Phase 17: one line fixed system dialog visibility)
3. CI/CD pipeline should be an early milestone, not a late addition -- every manual release is friction

### Cost Observations
- Model mix: primarily opus for execution, sonnet for planning
- All 5 plans completed in under 20 minutes total execution time
- Notable: Phase 19 (150 pattern expansion) completed in 3 minutes -- regex patterns are high-leverage

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v0.1.0 | 8 | 21 | Initial MVP, established GSD workflow |
| v0.1.1 | 3 | 6 | Fast follow-up, focused scope |
| v0.2.1 | 7 | 11 | Cross-platform port, parallel branch work |
| v0.2.4 | 4 | 5 | UX polish + infra, yolo mode with plan-check |

### Top Lessons (Verified Across Milestones)

1. Small focused phases execute faster and produce cleaner commits than large multi-concern phases
2. Capture-before-show pattern applies everywhere: get context before changing state
3. In-memory state with no disk persistence is almost always the right v1 choice for transient preferences
